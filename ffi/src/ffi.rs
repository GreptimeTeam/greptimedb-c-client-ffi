use crate::error::StatusCode;
use crate::row::{RowBuilder, Value};
use crate::{ensure_not_null, error, Client};
use greptimedb_client::api::v1::{Column, ColumnDataType};
use std::ffi;

#[no_mangle]
pub unsafe extern "C" fn new_row_builder(
    table_name: *const libc::c_char,
    res_ptr: *mut *const RowBuilder,
) -> libc::c_int {
    ensure_not_null!(table_name);
    let col_name = match std::ffi::CStr::from_ptr(table_name).to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!("Cannot convert table name, e: {:?}", e);
        }
    };

    *res_ptr = Box::into_raw(Box::new(RowBuilder::new(col_name.to_string())));
    StatusCode::Success as i32
}

#[no_mangle]
pub unsafe extern "C" fn add_column(
    row_builder: *mut RowBuilder,
    col_name: *const libc::c_char,
    data_type: libc::c_int,
    semantic_type: libc::c_int,
) -> libc::c_int {
    ensure_not_null!(row_builder);
    ensure_not_null!(col_name);

    let builder = unsafe { &mut *row_builder };
    let col_name = match std::ffi::CStr::from_ptr(col_name).to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!("Cannot convert field name, e: {:?}", e);
        }
    };

    builder.add_col(col_name.to_string(), data_type, semantic_type);
    StatusCode::Success as i32
}

#[no_mangle]
pub unsafe extern "C" fn add_row(
    row_builder: *mut RowBuilder,
    values: *const Value,
    value_len: libc::size_t,
) -> libc::c_int {
    ensure_not_null!(row_builder);
    ensure_not_null!(values);

    let builder = unsafe { &mut *row_builder };

    let values = std::slice::from_raw_parts(values, value_len);
    builder.add_row(values);
    StatusCode::Success as i32
}

#[no_mangle]
pub unsafe extern "C" fn new_client(
    database_name: *const libc::c_char,
    endpoint: *const libc::c_char,
    res_ptr: *mut *const Client,
) -> libc::c_int {
    ensure_not_null!(database_name);
    ensure_not_null!(endpoint);
    let database_name = match std::ffi::CStr::from_ptr(database_name).to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!("Cannot convert database name, e: {:?}", e);
        }
    };
    let endpoint = match std::ffi::CStr::from_ptr(endpoint).to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!("Cannot convert endpoint, e: {:?}", e);
        }
    };

    let client = Client::new(database_name.to_string(), endpoint.to_string()).unwrap();
    *res_ptr = Box::into_raw(Box::new(client));

    StatusCode::Success as i32
}

#[no_mangle]
pub unsafe extern "C" fn write_row(client: *const Client, row: *mut RowBuilder) -> libc::c_int {
    ensure_not_null!(client);
    ensure_not_null!(row);
    let client = unsafe { &*client };
    let row = unsafe { &mut *row };
    client.write_row(row);

    StatusCode::Success as i32
}
