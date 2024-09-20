pub struct Transaction{
    pub id:u64,
}
impl Transaction{
    pub fn to_string(&self) ->String{
        format!("{}", self.id)
    }
}