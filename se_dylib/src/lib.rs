mod base;

use se_rust::client::TcpClient;
use se_rust::server::TcpServer;
use std::ffi::{c_char, c_double, c_int, c_uchar, c_uint, c_void, CStr};

#[no_mangle]
pub unsafe extern "C" fn new_server() -> *mut c_void {
    let Ok(server) = TcpServer::new() else {
        return std::ptr::null_mut();
    };

    Box::into_raw(Box::new(server)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn s_send(com: *mut c_void, data: *const c_uchar, len: c_uint) -> c_int {
    base::send::<TcpServer>(com, data, len)
}

#[no_mangle]
pub unsafe fn s_receive(com: *mut c_void, buf: *mut c_uchar, len: c_uint) -> c_int {
    base::receive::<TcpServer>(com, buf, len)
}

#[no_mangle]
pub unsafe fn s_send_table(
    com: *mut c_void,
    table: *const c_double,
    row_num: c_uint,
    col_num: c_uint,
) -> c_int {
    base::send_table::<TcpServer>(com, table, row_num, col_num)
}

#[no_mangle]
pub unsafe fn s_receive_table(
    com: *mut c_void,
    table_buf: *mut c_double,
    len: c_uint,
    row_num: *mut c_uint,
    col_num: *mut c_uint,
) -> i32 {
    base::receive_table::<TcpServer>(com, table_buf, len, row_num, col_num)
}

#[no_mangle]
pub unsafe fn s_close(com: *mut c_void) {
    base::close::<TcpServer>(com)
}

#[no_mangle]
pub unsafe extern "C" fn new_client(server_address: *const c_char) -> *mut c_void {
    let Ok(server_address) = CStr::from_ptr(server_address).to_str() else {
        return std::ptr::null_mut();
    };

    let Ok(client) = TcpClient::new(server_address) else {
        return std::ptr::null_mut();
    };

    Box::into_raw(Box::new(client)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn c_send(com: *mut c_void, data: *const c_uchar, len: c_uint) -> c_int {
    base::send::<TcpClient>(com, data, len)
}

#[no_mangle]
pub unsafe fn c_receive(com: *mut c_void, buf: *mut c_uchar, len: c_uint) -> c_int {
    base::receive::<TcpClient>(com, buf, len)
}

#[no_mangle]
pub unsafe fn c_send_table(
    com: *mut c_void,
    table: *const c_double,
    row_num: c_uint,
    col_num: c_uint,
) -> c_int {
    base::send_table::<TcpClient>(com, table, row_num, col_num)
}

#[no_mangle]
pub unsafe fn c_receive_table(
    com: *mut c_void,
    table_buf: *mut c_double,
    len: c_uint,
    row_num: *mut c_uint,
    col_num: *mut c_uint,
) -> i32 {
    base::receive_table::<TcpClient>(com, table_buf, len, row_num, col_num)
}

#[no_mangle]
pub unsafe fn c_close(com: *mut c_void) {
    base::close::<TcpClient>(com)
}
