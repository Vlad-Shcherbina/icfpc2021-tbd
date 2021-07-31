use crate::util::{project_path, load_problem, all_problem_ids};
use crate::domain_model::Pose;


fn connect() -> Result<postgres::Client, postgres::Error> {
    let content = std::fs::read_to_string(project_path("data/db_pwd.txt")).unwrap();
    let client = postgres::Client::connect(&content, postgres::NoTls)?;
    Ok(client)
}

fn create_db() -> Result<(), postgres::Error> {
    let mut client = connect()?;
    client.batch_execute(
        "CREATE TYPE bonustype AS ENUM (
            'SUPERFLEX', 'WALLHACK', 'BREAK_A_LEG', 'GLOBALIST'
        );"
    )?;

    client.batch_execute(
        "CREATE TABLE bonuses (
            id      SERIAL PRIMARY KEY,
            src     integer NOT NULL,
            dest    integer NOT NULL,
            type    bonustype NOT NULL
        );"
    )?;

    client.batch_execute(
        "CREATE TABLE solutions (
            id          SERIAL PRIMARY KEY,
            problem     integer NOT NULL,
            text        JSON NOT NULL,
            dislikes    integer NOT NULL,
            bonus_used  bonustype
        );"
    )?;

    client.batch_execute(
        "CREATE TABLE bonuses_unlocked (
            id          SERIAL PRIMARY KEY,
            solution    integer NOT NULL REFERENCES solutions (id) ON DELETE CASCADE,
            dest        integer NOT NULL,
            type        bonustype NOT NULL
        );"
    )?;

    Ok(())
}

pub fn update_bonus_graph() -> Result<(), postgres::Error> {
    let mut client = connect()?;
    client.batch_execute(
        "DELETE FROM bonuses"
    )?;
    
    for id in all_problem_ids() {
        let p = load_problem(id);
        for b in p.bonuses {
            let smth = client
                    .prepare_typed("INSERT INTO bonuses (src, dest, type) VALUES ($1, $2, $3::bonustype);",
                                    &[postgres::types::Type::INT4,
                                      postgres::types::Type::INT4,
                                      postgres::types::Type::TEXT])?;
            client.execute(&smth, &[&id, &b.problem, &b.bonus.to_string()])?;
        }
    }
    Ok(())
}

crate::entry_point!("db_setup", db_setup, _EP1);
pub fn db_setup() {
    match create_db() {
        Ok(_) => println!("Database set up successfully"),
        Err(a) => println!("ERROR while setting up: {}", a),
    }
}

crate::entry_point!("db_update_problems", update_problems, _EP2);
pub fn update_problems() {
    match update_bonus_graph() {
        Ok(_) => println!("Bonus graph in DB updated successfully"),
        Err(a) => println!("ERROR while updating: {}", a),
    }
}

#[allow(dead_code)]
pub fn update_validator() -> Result<(), postgres::Error> {
    let mut client = connect()?;
    for problem_id in all_problem_ids() {
        let problem = load_problem(problem_id);
        for row in client.query("SELECT * FROM solutions WHERE problem = $1;", &[&problem_id])? {
            let problem_id: i64 = row.get("problem");
            let text: String = row.get("text");
            let pose: Pose = serde_json::from_str(&text).unwrap();
            if !crate::checker::check_pose(&problem, &pose).valid {
                let sol_id : i64 = row.get("id");
                client.execute("DELETE * FROM solutions WHERE id = $1;", &[&sol_id])?;
                let filename = format!("output/invalidated/{}-{}.json", problem_id, sol_id);

                std::fs::write(project_path(&filename), text).unwrap();
                println!("Invalidated solution for problem {} (written to {})", problem_id, &filename);
            }
        }    
    }

    Ok(())
}


// Function assumes that solution has already been validated.
pub fn write_valid_solution_to_db(problem_id: i64, pose: &Pose, dislikes: i64) 
                -> Result<(), postgres::Error> {
    let mut client = connect()?;
    if pose.bonuses.is_empty() {
        client.execute(
            "INSERT INTO solution (problem, text, dislikes) VALUES ($1, $2, $3);",
            &[&problem_id, &serde_json::to_vec(pose).unwrap(), &dislikes]
        )?;
    }
    else {
        client.execute(
            "INSERT INTO solution (problem, text, dislikes, bonus) VALUES ($1, $2, $3, $4);",
            &[&problem_id, &serde_json::to_vec(pose).unwrap(), &dislikes, &pose.bonuses[0].bonus.to_string()]
        )?;
    }

    Ok(())
}
