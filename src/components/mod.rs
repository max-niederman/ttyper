pub mod result;
pub mod test;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Screen {
    Test,
    Results,
}
