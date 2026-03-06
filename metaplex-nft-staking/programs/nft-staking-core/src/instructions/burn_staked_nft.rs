use crate::errors::StakingError;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to_checked, Mint, MintToChecked, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        BurnV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{
        Attribute, Attributes, BurnDelegate, FreezeDelegate, Plugin, PluginType, UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

const SECONDS_PER_DAY: i64 = 86400;
#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: NFT account will be checked by the mpl core program
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> BurnStakedNft<'info> {
    pub fn burn_staked_nft(&mut self, bumps: BurnStakedNftBumps) -> Result<()> {
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(
            base_asset.owner == self.user.key(),
            StakingError::InvalidOwner
        );
        require!(
            base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()),
            StakingError::InvalidAuthority
        );
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(
            base_collection.update_authority == self.update_authority.key(),
            StakingError::InvalidAuthority
        );

        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        let current_timestamp = Clock::get()?.unix_timestamp;

        let fetched_attritbute_list = match fetch_plugin::<BaseAssetV1, Attributes>(
            &self.nft.to_account_info(),
            PluginType::Attributes,
        ) {
            Err(_) => return Err(StakingError::NotStaked.into()),
            Ok((_, attributes, _)) => attributes,
        };

        let mut attribute_list: Vec<Attribute> =
            Vec::with_capacity(fetched_attritbute_list.attribute_list.len());
        let mut staked_value: Option<&str> = None;
        let mut staked_at_value: Option<&str> = None;

        for attribute in &fetched_attritbute_list.attribute_list {
            match attribute.key.as_str() {
                "staked" => {
                    staked_value = Some(&attribute.value);
                    attribute_list.push(Attribute {
                        key: "staked".to_string(),
                        value: "false".to_string(),
                    });
                }
                "staked_at" => {
                    staked_at_value = Some(&attribute.value);
                    attribute_list.push(Attribute {
                        key: "staked_at".to_string(),
                        value: "0".to_string(),
                    });
                }
                _ => {
                    attribute_list.push(attribute.clone());
                }
            }
        }

        require!(staked_value == Some("true"), StakingError::NotStaked);

        let staked_at_timestamp = staked_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;
        let elapsed_seconds = current_timestamp
            .checked_sub(staked_at_timestamp)
            .ok_or(StakingError::InvalidTimestamp)?;
        let staked_time_days = elapsed_seconds
            .checked_div(SECONDS_PER_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;

        // unfreeze
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(&[signer_seeds])?;

        // burn
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .authority(Some(&self.user.to_account_info()))
            .payer(&self.user.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke()?;

        // Updating Collection
        match fetch_plugin::<BaseCollectionV1, Attributes>(
            &self.collection.to_account_info(),
            PluginType::Attributes,
        ) {
            Ok((_, fetched_attributes, _)) => {
                let mut collection_attribute_list: Vec<Attribute> = Vec::new();
                for attribute in fetched_attributes.attribute_list {
                    if attribute.key == "total_staked" {
                        let current: u64 = attribute.value.parse().unwrap_or(0);
                        collection_attribute_list.push(Attribute {
                            key: "total_staked".to_string(),
                            value: (current - 1).to_string(),
                        });
                    } else {
                        collection_attribute_list.push(attribute);
                    }
                }
                UpdateCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .collection(&self.collection.to_account_info())
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(Attributes {
                        attribute_list: collection_attribute_list,
                    }))
                    .invoke_signed(&[signer_seeds])?;
            }
            Err(_) => return Err(StakingError::CollectionAttributeNotFound.into()),
        };
        // Calculate rewards to the user
        let amount = (staked_time_days as u64)
            .checked_mul(self.config.points_per_stake as u64)
            .ok_or(StakingError::Overflow)?;

        let total_amount_with_burn_rewards = amount
            .checked_add(self.config.burn_rewards as u64)
            .ok_or(StakingError::Overflow)?;

        let config_seeds = &[
            b"config",
            collection_key.as_ref(),
            &[self.config.config_bump],
        ];
        let config_signer_seeds = &[&config_seeds[..]];

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintToChecked {
            mint: self.rewards_mint.to_account_info(),
            to: self.user_rewards_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, config_signer_seeds);
        mint_to_checked(
            cpi_ctx,
            total_amount_with_burn_rewards,
            self.rewards_mint.decimals,
        )?;
        Ok(())
    }
}
