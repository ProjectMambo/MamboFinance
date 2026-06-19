mod user;
use user::*;
mod simulation;
use simulation::*;

fn main() -> Result<(), UserError> {
    println!("\n--------------------------------------------------\n");

    let user = User::new_in_memory("TEST")?;
    simulate_user_activity(&user)?;

    println!("\n--------------------------------------------------\n");
    Ok(())
}
