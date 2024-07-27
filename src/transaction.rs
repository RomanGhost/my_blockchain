pub struct Transaction{
    pub(crate) id:u64,
}
impl Transaction{
    pub fn to_string(&self) ->String{
        format!("{}", self.id)
    }
}