import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TuktukEscrow } from "../target/types/tuktuk_escrow";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  init as initTuktuk,
  taskKey,
  taskQueueAuthorityKey,
} from "@helium/tuktuk-sdk";
import { createMint, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("tuktuk-escrow",() => {
  // Configure the client to use the local cluster.
  const provider=anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.tuktukEscrow as Program<TuktukEscrow>;
  const connection=provider.connection;
  const providerWallet=provider.wallet as anchor.Wallet;
  const payer=providerWallet.payer;
  console.log("RPC:", provider.connection.rpcEndpoint);

  let mintA:PublicKey;
  let mintB:PublicKey;
  let makerAtaA:any;

  const seed = new anchor.BN(Math.floor(Math.random() * 1000000));
  const seedBuf=Buffer.alloc(8);
  seedBuf.writeBigUInt64LE(BigInt(seed.toString()));
  let escrow:PublicKey;
  let vault:PublicKey;

  let tqAuthPda:PublicKey;
  let tqAuthInfo:any;

  const TUKTUK_PROGRAM_ID = new PublicKey(
  "tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA"
);
  const taskQueue = new PublicKey("CMreFdKxT5oeZhiX8nWTGz9PtXM1AMYTh6dGR2UzdtrA");
  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("queue_authority")], program.programId)[0];
  const taskQueueAuthority = taskQueueAuthorityKey(taskQueue, queueAuthority)[0];
  
before(async()=>{
  const tuktukProgram = await initTuktuk(provider);
  console.log(
  "Tuktuk instructions:",
  Object.keys(tuktukProgram.methods)
);


  mintA=await createMint(connection,payer,payer.publicKey,null,6);
  mintB =await createMint(connection,payer,payer.publicKey,null,6);

  console.log("mint A and B:",mintA,mintB);
  [escrow]=PublicKey.findProgramAddressSync([Buffer.from("escrow"),payer.publicKey.toBuffer(),seedBuf],program.programId);
  console.log("Escrow PDA",escrow);
  vault=getAssociatedTokenAddressSync(mintA,escrow,true);
  console.log("Vault",vault);
   
  makerAtaA = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mintA,
    payer.publicKey
  );
  console.log("Maker ATA",makerAtaA);
  await mintTo(
    connection,
    payer,
    mintA,
    makerAtaA.address,
    payer,
    1_000_000_000
  );

  
  tqAuthPda = taskQueueAuthorityKey(taskQueue, queueAuthority)[0];
  console.log("Auth PDA",tqAuthPda);
  tqAuthInfo = await connection.getAccountInfo(tqAuthPda);
  
    if (!tqAuthInfo) {
    console.log("Registering...");
    const regTx = await tuktukProgram.methods
      .addQueueAuthorityV0()
      .accounts({
        payer: payer.publicKey,
        queueAuthority,
        taskQueue: taskQueue,
      })
      .rpc({ skipPreflight: true, commitment: "confirmed" });
    console.log("Registered — tx:", regTx);
  } else {
    console.log("Already registered.");
  }
});
  
  it("Make", async () => {
    const receive=new anchor.BN(1_0000_000);
    const deposit=new anchor.BN(1_000);
    
    const makeTx=await program.methods.make(seed,deposit,receive).accounts({
      maker:payer.publicKey,
      mintA,
      mintB,
    })
    .rpc({ skipPreflight: true, commitment: "confirmed" });
    
    console.log("Make completed {}",makeTx);
  });
  
  it("Schedule", async () => {
  let tuktukProgram = await initTuktuk(provider);
  let taskId=1;
  const taskIdBuf = Buffer.alloc(2);
  taskIdBuf.writeUInt16LE(taskId);

  const [taskAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("task"), taskQueue.toBuffer(), taskIdBuf],
    TUKTUK_PROGRAM_ID
  );
const [tqAuthorityPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("task_queue_authority"),
      taskQueue.toBuffer(),
      queueAuthority.toBuffer(),
    ],
    TUKTUK_PROGRAM_ID
  );
  
  
  try {
    const tx = await program.methods.schedule(taskId).accountsPartial({
      maker: payer.publicKey,
      mintA,
      escrow: escrow,
      vault,
      task: taskAccount,
      taskQueue,
      taskQueueAuthority:tqAuthorityPda,
      tuktukProgram:TUKTUK_PROGRAM_ID
    }).rpc();
    
    console.log("Schedule completed", tx);
  } catch (error) {
    console.log("\nFull error:", error);
    if (error.logs) {
      console.log("\nTransaction logs:");
      error.logs.forEach(log => console.log(log));
    }
    throw error;
  }
});
});