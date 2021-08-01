use serde_json::de::Read;

use crate::util::{project_path, load_problem, all_problem_ids};
use crate::domain_model::Pose;


pub fn connect() -> Result<postgres::Client, postgres::Error> {
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
pub fn update_validator(client: &mut postgres::Client) -> Result<(), postgres::Error> {
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
pub fn write_valid_solution_to_db(client: &mut postgres::Client,
                problem_id: i32, pose: &Pose, dislikes: i64) 
                -> Result<(), postgres::Error> {
    if pose.bonuses.is_empty() {
        client.execute(
            "INSERT INTO solutions (problem, text, dislikes) VALUES ($1, $2, $3);",
            &[&problem_id, &serde_json::to_value(pose).unwrap(), &(dislikes as i32)]
        )?;
    }
    else {
        client.execute(
            "INSERT INTO solutions (problem, text, dislikes, bonus) VALUES ($1, $2, $3, $4);",
            &[&problem_id, &serde_json::to_vec(pose).unwrap(), &dislikes, &pose.bonuses[0].bonus.to_string()]
        )?;
    }

    Ok(())
}


pub fn get_solutions_stats_by_problem(client: &mut postgres::Client, problem_id: i32) 
                -> Result<Vec<(i32, i32)>, postgres::Error> {
    let res = client.query("SELECT id, dislikes FROM solutions WHERE (problem = $1)", &[&problem_id])?;
    Ok(res.iter().map(|r| (r.get("id"), r.get("dislikes"))).collect())
}


pub fn get_solution_by_id(client: &mut postgres::Client, solution_id: i32)
                -> Result<Option<Pose>, postgres::Error> {
    let res = client.query("SELECT text FROM solutions WHERE (id = $1)", &[&solution_id])?;
    match res.len() {
        0 => Ok(None),
        1 => Ok(Some(serde_json::from_value(res[0].get("text")).unwrap())),
        _ => panic!(),
    }
}
