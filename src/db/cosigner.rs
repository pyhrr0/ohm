use std::error::Error;

use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use super::models::{Cosigner, CosignerType, NewCosigner};
use super::schema::cosigner::table;
use crate::grpc::pb;

pub fn store_cosigner(
    conn: &mut SqliteConnection,
    cosigner: &pb::Cosigner,
    type_: CosignerType,
) -> Result<Cosigner, Box<dyn Error>> {
    let cosigner_id = Uuid::new_v4();

    Ok(diesel::insert_into(table)
        .values(&NewCosigner {
            uuid: &cosigner_id.to_string(),
            type_,
            email_address: &cosigner.email_address,
            public_key: &cosigner.public_key,
            creation_time: Utc::now().naive_local(),
        })
        .get_result(conn)?)
}
