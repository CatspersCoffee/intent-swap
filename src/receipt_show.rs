#![allow(warnings)]

use fuel_vm::prelude::*;
use fuels::types::{
    Bits256, B512, EvmAddress, Bytes,
};
use fuels::programs::responses::CallResponse;



/// just an ugly response decoder and dumper.
pub fn log_shower<T>(fr: CallResponse<T>)
where
    T: std::fmt::Debug,
{

    let logs_res = fr.decode_logs();
    // println!("logs_res:\n{:?}", logs_res);

    let log1_string = fr.decode_logs_with_type::<String>();

    let mut values_string_arr: Vec<String> = Vec::new();
    match log1_string {
        Ok(values) => {
            let g = values.len();
            println!("number of String's = {}", g);
            for i in 0..g{
                values_string_arr.push(values[i].clone());
            }


        }
        Error => {

        }
    };

    for (i, v) in values_string_arr.iter().enumerate() {
        match i {
            0 => println!("log: String[{:02}] --> .. : {}", i, *v),
            _ => println!("log: String[{:02}] --> .. : {}", i, *v),
        }
    }

}

pub fn hex_print_bytes(bytes: &Vec<u8>) -> String{
    hex::encode(bytes)
}


/*
pub mod receipt_sniffer {
    use super::*;
    use fuels::prelude::*;
    use fuels::types::Bytes32;
    use fuels::types::errors::transaction::Reason;
    use fuels::types::tx_status::TxStatus;
    use tokio::time::{sleep, Duration};

    #[derive(Debug)]
    pub enum TxPollError {
        Timeout {
            tx_id: Bytes32,
            duration: Duration,
        },
        TransactionError(Error),
    }

    #[derive(Debug)]
    pub enum TxStatusResult {
        Ok {
            receipts: Vec<Receipt>
        },
        Err {
            error: TxPollError,
            status: TxStatus,
            elapsed_time: Duration
        }
    }

    /// Polls transaction status until it succeeds or fails
    /// Returns TxStatusResult which contains either receipts on success
    /// or detailed error information on failure
    pub async fn wait_for_tx_success(
        provider: &Provider,
        tx_id: &Bytes32,
        poll_interval_ms: u64,
        timeout_secs: u64,
    ) -> TxStatusResult {
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let poll_interval = Duration::from_millis(poll_interval_ms);

        loop {
            let elapsed = start_time.elapsed();
            if elapsed > timeout {
                return TxStatusResult::Err {
                    error: TxPollError::Timeout {
                        tx_id: tx_id.clone(),
                        duration: elapsed,
                    },
                    status: TxStatus::Submitted,
                    elapsed_time: elapsed,
                };
            }

            match provider.tx_status(&tx_id).await {
                Ok(status) => match status {
                    TxStatus::Success { receipts } => {
                        return TxStatusResult::Ok { receipts }
                    }
                    TxStatus::Revert { reason, revert_id, receipts } => {
                        return TxStatusResult::Err {
                            error: TxPollError::TransactionError(Error::Transaction(
                                Reason::Reverted {
                                    reason: reason.clone(),
                                    revert_id,
                                    receipts: receipts.clone(),
                                }
                            )),
                            status: TxStatus::Revert {
                                reason,
                                revert_id,
                                receipts,
                            },
                            elapsed_time: elapsed,
                        };
                    }
                    TxStatus::SqueezedOut { reason } => {
                        return TxStatusResult::Err {
                            error: TxPollError::TransactionError(Error::Transaction(
                                Reason::SqueezedOut(reason.clone())
                            )),
                            status: TxStatus::SqueezedOut { reason },
                            elapsed_time: elapsed,
                        };
                    }
                    TxStatus::Submitted => {
                        sleep(poll_interval).await;
                        continue;
                    }
                },
                Err(e) => {
                    // Check if the error is "status not found"
                    if e.to_string().contains("status not found for transaction") {
                        // This is expected while waiting for the transaction to be processed
                        sleep(poll_interval).await;
                        continue;
                    }

                    // For other errors, return the error
                    return TxStatusResult::Err {
                        error: TxPollError::TransactionError(e),
                        status: TxStatus::Submitted,
                        elapsed_time: elapsed,
                    };
                }
            }
        }
    }
}
*/

pub mod receipt_sniffer {
    use super::*;
    use fuels::prelude::*;
    use fuels::types::Bytes32;
    use fuels::types::errors::transaction::Reason;
    use fuels::types::tx_status::TxStatus;
    use tokio::time::{sleep, Duration};

    #[derive(Debug)]
    pub enum TxPollError {
        Timeout {
            tx_id: Bytes32,
            duration: Duration,
        },
        TransactionError(Error),
    }

    #[derive(Debug)]
    pub enum TxStatusResult {
        Ok {
            receipts: Vec<Receipt>,
            elapsed_time: Duration
        },
        Err {
            error: TxPollError,
            status: TxStatus,
            elapsed_time: Duration
        }
    }

    /// Polls transaction status until it succeeds or fails
    /// Returns TxStatusResult which contains either receipts on success
    /// or detailed error information on failure, along with timing information
    pub async fn wait_for_tx_success(
        provider: &Provider,
        tx_id: &Bytes32,
        poll_interval_ms: u64,
        timeout_secs: u64,
    ) -> TxStatusResult {
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let poll_interval = Duration::from_millis(poll_interval_ms);

        loop {
            let elapsed = start_time.elapsed();
            if elapsed > timeout {
                return TxStatusResult::Err {
                    error: TxPollError::Timeout {
                        tx_id: tx_id.clone(),
                        duration: elapsed,
                    },
                    status: TxStatus::Submitted,
                    elapsed_time: elapsed,
                };
            }

            match provider.tx_status(&tx_id).await {
                Ok(status) => match status {
                    TxStatus::Success { receipts } => {
                        return TxStatusResult::Ok {
                            receipts,
                            elapsed_time: start_time.elapsed()
                        }
                    }
                    TxStatus::Revert { reason, revert_id, receipts } => {
                        return TxStatusResult::Err {
                            error: TxPollError::TransactionError(Error::Transaction(
                                Reason::Reverted {
                                    reason: reason.clone(),
                                    revert_id,
                                    receipts: receipts.clone(),
                                }
                            )),
                            status: TxStatus::Revert {
                                reason,
                                revert_id,
                                receipts,
                            },
                            elapsed_time: elapsed,
                        };
                    }
                    TxStatus::SqueezedOut { reason } => {
                        return TxStatusResult::Err {
                            error: TxPollError::TransactionError(Error::Transaction(
                                Reason::SqueezedOut(reason.clone())
                            )),
                            status: TxStatus::SqueezedOut { reason },
                            elapsed_time: elapsed,
                        };
                    }
                    TxStatus::Submitted => {
                        sleep(poll_interval).await;
                        continue;
                    }
                },
                Err(e) => {
                    // Check if the error is "status not found"
                    if e.to_string().contains("status not found for transaction") {
                        // This is expected while waiting for the transaction to be processed
                        sleep(poll_interval).await;
                        continue;
                    }

                    // For other errors, return the error
                    return TxStatusResult::Err {
                        error: TxPollError::TransactionError(e),
                        status: TxStatus::Submitted,
                        elapsed_time: elapsed,
                    };
                }
            }
        }
    }
}