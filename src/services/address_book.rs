use crate::models::address_book::AddressBook;
use crate::repositories::address_book::AddressBookDao;
use crate::serialize::error::AppError;

#[derive(Clone, Debug)]
pub struct AddressBookSrv {
    address_book_dao: AddressBookDao,
}

impl AddressBookSrv {
    pub fn new(address_book_dao: AddressBookDao) -> Self {
        AddressBookSrv {
            address_book_dao: address_book_dao.clone(),
        }
    }

    pub async fn get_address_list(&self, address: &String) -> Result<Vec<AddressBook>, AppError> {
        self.address_book_dao
            .get_address_books(&address.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }
}
