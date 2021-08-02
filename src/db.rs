// use serde_json::de::Read;

use crate::util::{project_path, load_problem, all_problem_ids};
use crate::domain_model::{Pose, ProblemTgtBonus};


pub fn connect() -> Result<postgres::Client, postgres::Error> {
    let content = std::fs::read_to_string(project_path("data/db_pwd.txt")).unwrap();
    let client = postgres::Client::connect(&content, postgres::NoTls)?;
    Ok(client)
}

fn create_db() -> Result<(), postgres::Error> {
    let mut client = connect()?;
    client.batch_execute(
        "DROP SCHEMA public CASCADE;
         CREATE SCHEMA public;"
    )?;

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
            dislikes    int8 NOT NULL,
            bonus       bonustype,
            solver      varchar(20),
            time        timestamp
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
    let mut transaction = client.transaction()?;
    transaction.batch_execute(
        "DELETE FROM bonuses"
    )?;
    
    for id in all_problem_ids() {
        let p = load_problem(id);
        for b in p.bonuses {
            transaction.execute(
                "INSERT INTO bonuses (src, dest, type) VALUES ($1, $2, $3);",
                &[&id, &b.problem, &b.bonus])?;
        }
    }
    transaction.commit()?;
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
pub fn write_valid_solution_to_db(
                client: &mut postgres::Client,
                problem_id: i32, 
                pose: &Pose, 
                dislikes: i64,
                solver: &str)
                -> Result<(), postgres::Error> {

    let bonus = match pose.bonuses.len() {
        0 => None,
        1 => Some(pose.bonuses[0].bonus),
        _ => panic!("too many bonuses: {:?}", pose),
    };
    let mut transaction = client.transaction()?;

    let solution_id: i32 = transaction.query_one(
        "INSERT INTO solutions (problem, text, dislikes, bonus, solver, time)
                        VALUES ($1, $2, $3, $4, $5, $6) RETURNING id;", 
        &[&problem_id, &serde_json::to_value(pose).unwrap(),
          &dislikes, &bonus, &solver, &std::time::SystemTime::now()]
    )?.get("id");

    let problem = load_problem(problem_id);
    for bonus in problem.bonuses {
        if pose.vertices.iter().any(|v| *v == bonus.position) {

            transaction.execute(
                "INSERT INTO bonuses_unlocked (solution, dest, type)
                    VALUES ($1, $2, $3);",
                &[&solution_id, &bonus.problem, &bonus.bonus]
            )?;
        }
    }
    transaction.commit()?;

    Ok(())
}


pub fn get_solutions_stats_by_problem(client: &mut postgres::Client, problem_id: i32) 
                -> Result<Vec<(i32, i64)>, postgres::Error> {
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

pub fn get_target_bonuses_by_problem(client: &mut postgres::Client, problem_id: i32) 
-> Result<Vec<ProblemTgtBonus>, postgres::Error> {
    let res = client.query("SELECT src, type FROM bonuses WHERE dest = $1;", &[&problem_id])?;
    Ok(res.iter().map(|r| {
        ProblemTgtBonus {
            bonus: r.get("type"),
            from_problem: r.get("src")
        }
    }).collect())    
}
