use se_rust::comm::Communicator;
use std::ffi::{c_double, c_int, c_uchar, c_uint, c_void};
use std::mem::forget;
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub(crate) unsafe fn send<C: Communicator>(
    com: *mut c_void,
    data: *const c_uchar,
    len: c_uint,
) -> c_int {
    let mut com = Box::from_raw(com as *mut C);
    let data = from_raw_parts(data, len as usize);

    let Ok(_) = com.send(data) else {
        return -1;
    };

    // println!("sended: {:?}", data);

    forget(com);

    0
}

pub(crate) unsafe fn receive<C: Communicator>(
    com: *mut c_void,
    buf: *mut c_uchar,
    len: c_uint,
) -> c_int {
    let mut com = Box::from_raw(com as *mut C);
    let Ok(data) = com.receive() else {
        return -1;
    };

    if len < data.len() as u32 {
        return -1;
    }

    let buf = from_raw_parts_mut(buf, len as usize);

    for (i, b) in data.iter().enumerate() {
        buf[i] = *b;
    }

    // println!("received: {:?}", data);

    forget(com);

    data.len() as i32
}

pub(crate) unsafe fn send_table<C: Communicator>(
    com: *mut c_void,
    table: *const c_double,
    row_num: c_uint,
    col_num: c_uint,
) -> c_int {
    let mut com = Box::from_raw(com as *mut C);

    let table = from_raw_parts(table, (row_num * col_num) as usize);

    let table = table
        .chunks(col_num as usize)
        .map(|row| row.to_vec())
        .collect::<Vec<Vec<f64>>>();

    let Ok(_) = Communicator::send_table(com.as_mut(), table) else {
        return -1;
    };

    forget(com);

    0
}

pub(crate) unsafe fn receive_table<C: Communicator>(
    com: *mut c_void,
    table_buf: *mut c_double,
    len: c_uint,
    row_num: *mut c_uint,
    col_num: *mut c_uint,
) -> i32 {
    let mut com = Box::from_raw(com as *mut C);

    let Ok(table) = Communicator::receive_table(com.as_mut()) else {
        return -1;
    };

    let rn = table.len();
    let cn = table.get(0).map(|r| r.len()).unwrap_or(0);

    if len < (rn * cn) as u32 {
        return -1;
    }

    let table_buf = from_raw_parts_mut(table_buf, len as usize);

    for (i, row) in table.iter().enumerate() {
        for (j, b) in row.iter().enumerate() {
            table_buf[i * row.len() + j] = *b;
        }
    }

    *row_num = rn as u32;
    *col_num = cn as u32;

    forget(com);

    0
}

pub(crate) unsafe fn close<C: Communicator>(com: *mut c_void) {
    let _com = Box::from_raw(com as *mut C);
    // drop(com.as_mut());
}

/*
type SendSig = unsafe extern "C" fn(*mut c_void, *const u8, u32) -> i32;
type RecvSig = unsafe extern "C" fn(*mut c_void, *mut u8, u32) -> i32;
type SendTableSig = unsafe extern "C" fn(*mut c_void, *const f64, u32, u32) -> i32;
type RecvTableSig = unsafe extern "C" fn(*mut c_void, *mut f64, u32, *mut u32, *mut u32) -> i32;
type ScenarioSig = unsafe extern "C" fn(*mut c_void, SendSig, RecvSig, SendTableSig, RecvTableSig);

unsafe fn base<C: Communicator>(
    com: *mut c_void,
    scenario: *const c_void,
    // scenario: fn(SendSig, RecvSig, SendTableSig, RecvTableSig),
) {
    let scenario: &ScenarioSig = &*(scenario as *const ScenarioSig);

    scenario(
        com,
        send::<C>,
        receive::<C>,
        send_table::<C>,
        receive_table::<C>,
    );
}
*/
