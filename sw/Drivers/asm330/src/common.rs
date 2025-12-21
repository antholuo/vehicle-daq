/*
* Created date: 12/20/25
* File Description: common definitions used across files
*/

use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum Asm330Error {
    #[error("Data not ready for read")]
    DataNotReadyError(),
    #[error("Spi error occured")]
    SpiError(),
}
