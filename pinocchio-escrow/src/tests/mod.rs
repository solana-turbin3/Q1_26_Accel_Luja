#[cfg(test)]
mod helper;
#[cfg(test)]
mod test {
    use crate::tests::helper::{make_ix, setup};

    #[test]
    pub fn make_instruction() {
        make_ix();
    }
}
