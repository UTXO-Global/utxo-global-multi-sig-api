use crate::models::address_book::AddressBook;
use crate::repositories::address_book::AddressBookDao;
use crate::serialize::address_book::AddressBookReq;
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

    pub async fn update_address(
        &self,
        user_address: &String,
        req: AddressBookReq,
    ) -> Result<AddressBook, AppError> {
        let address_book = self
            .address_book_dao
            .get_address(user_address, &req.clone().signer_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        if address_book.is_none() {
            return Err(AppError::new(500).message("Address not found."));
        }

        let mut info = address_book.unwrap();

        match self
            .address_book_dao
            .update_address(user_address, req.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            true => {
                info.signer_name = req.clone().signer_name;
                Ok(info)
            }
            false => Err(AppError::new(500).message("Update address failed")),
        }
    }

    pub async fn add_address(
        &self,
        user_address: &String,
        req: AddressBookReq,
    ) -> Result<AddressBook, AppError> {
        let address_book = self
            .address_book_dao
            .get_address(user_address, &req.clone().signer_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        if !address_book.is_none() {
            return Ok(address_book.unwrap());
        }

        match self
            .address_book_dao
            .add_address(user_address, &req.signer_address, &req.signer_name)
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(AppError::new(500).message(&err.to_string())),
        }
    }
}
