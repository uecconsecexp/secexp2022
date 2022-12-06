use crate::comm::Communicator;
use anyhow::Result;
use std::io::{Read, Write};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Matrix {
    data: Vec<Vec<String>>,
}

pub(crate) fn send_table<C: Communicator<impl Write, impl Read> + ?Sized>(
    comm: &mut C,
    table: Vec<Vec<f64>>,
) -> Result<()> {
    if table.len() == 0 {
        return Err(anyhow::anyhow!("Invalid matrix"));
    }

    let data = table
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|x| format!("{:e}", x))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let matrix = Matrix { data };

    let data = serde_json::to_string(&matrix)?;
    comm.send(data.as_bytes())?;

    Ok(())
}

pub(crate) fn receive_table<C: Communicator<impl Write, impl Read> + ?Sized>(
    comm: &mut C,
) -> Result<Vec<Vec<f64>>> {
    let data = comm.receive()?;
    let matrix = serde_json::from_slice::<Matrix>(&data)?;

    let data = matrix
        .data
        .iter()
        .map(|row| {
            row.iter()
                .map(|x| {
                    x.parse::<f64>()
                        .map_err(|_| anyhow::anyhow!("Invalid number"))
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(data)
}
