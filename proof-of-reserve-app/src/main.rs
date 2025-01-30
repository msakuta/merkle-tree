use rocket::State;
use rocket::serde::{Serialize, json::Json};
use merkle_root_lib;

#[macro_use]
extern crate rocket;

#[get("/proof")]
fn proof_all_users(state: &State<AppState>) -> String {
    merkle_root_lib::compute(
        "ProofOfReserve_Branch",
        "ProofOfReserve_Leaf",
         &state.users_data)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct MerkleProof {
    user_balance: u32,
    proof: Vec<(String, u8)>,
}

#[get("/proof/<user_id>")]
fn proof_by_user_id(state: &State<AppState>, user_id: &str) -> Json<MerkleProof> {

    Json(MerkleProof {
        user_balance: 100,
        proof: vec![],
    })
}

struct AppState {
    users_data: Vec<String>,
}

#[launch]
fn rocket() -> _ {
    let users_data = vec![
        "(1,1111)".to_string(),
        "(2,2222)".to_string(),
        "(3,3333)".to_string(),
        "(4,4444)".to_string(),
        "(5,5555)".to_string(),
        "(6,6666)".to_string(),
        "(7,7777)".to_string(),
        "(8,8888)".to_string(),
    ];

    rocket::build()
        .manage(AppState { users_data })
        .mount("/", routes![proof_all_users, proof_by_user_id])
}
