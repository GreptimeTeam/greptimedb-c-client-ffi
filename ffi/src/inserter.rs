use crate::error::CreateStreamInserterSnafu;
use crate::{debug, error};
use greptimedb_client::api::v1::InsertRequest;
use greptimedb_client::StreamInserter;
use snafu::ResultExt;

pub struct Inserter {
    inner: Option<StreamInserter>,
    insert_request_receiver: tokio::sync::mpsc::Receiver<InsertRequest>,
}

impl Inserter {
    pub fn new(
        db_name: String,
        grpc_endpoint: String,
        insert_request_receiver: tokio::sync::mpsc::Receiver<InsertRequest>,
    ) -> error::Result<Self> {
        let grpc_client = greptimedb_client::Client::with_urls(vec![&grpc_endpoint]);
        let client = greptimedb_client::Database::new_with_dbname(db_name, grpc_client);

        let stream_inserter = client
            .streaming_inserter_with_channel_size(1024)
            .context(CreateStreamInserterSnafu { grpc_endpoint })?;

        Ok(Self {
            inner: Some(stream_inserter),
            insert_request_receiver,
        })
    }

    pub async fn run(&mut self) {
        let stream_inserter = self.inner.as_mut().unwrap();

        while let Some(request) = self.insert_request_receiver.recv().await {
            println!("Request: {:?}", request);
            if let Err(e) = stream_inserter.insert(vec![request]).await {
                error!("Failed to send requests to database, error: {:?}", e);
                break;
            } else {
                println!("Successfully write request");
            }
        }

        match self.inner.take().unwrap().finish().await {
            Ok(rows) => debug!("Stream inserter finished after inserted {} rows", rows),
            Err(e) => error!("Failed to finish the stream inserter, error {:?}", e),
        }
    }
}

// pub fn maybe_split_insert_request(
//     req: InsertRequest,
// ) -> Box<dyn Iterator<Item = InsertRequest> + Send> {
//     const BATCH_SIZE: u32 = 1024;
//     if req.row_count > BATCH_SIZE {
//         let chunks = chunks(req.row_count as usize, BATCH_SIZE as usize);
//         debug!("Splitting request into {}", chunks.len());
//
//         let iter = chunks.into_iter().map(move |(lower, upper)| {
//             let mut columns = Vec::with_capacity(req.columns.len());
//
//             for col in &req.columns {
//                 let values = col.values.as_ref().map(|v| take_values(v, lower, upper));
//
//                 columns.push(Column {
//                     column_name: col.column_name.clone(),
//                     semantic_type: col.semantic_type,
//                     values,
//                     null_mask: col.null_mask.clone(),
//                     datatype: col.datatype,
//                 });
//             }
//             InsertRequest {
//                 table_name: req.table_name.clone(),
//                 columns,
//                 row_count: (upper - lower) as u32,
//                 region_number: 0,
//             }
//         });
//         Box::new(iter)
//     } else {
//         Box::new(std::iter::once(req))
//     }
// }
