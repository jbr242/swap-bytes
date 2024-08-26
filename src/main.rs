mod back_end;
use back_end::behaviour;
use back_end::utils;
use back_end::swarm_builder;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    swarm_builder::start_swarm_builder().await
}


