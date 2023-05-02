extern crate cxx;

extern crate lazy_static;
extern crate regex;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("twsearch/src/cpp/rustapi.h");
        fn rust_arg(s: &str);
        fn rust_setksolve(s: &str);
        // fn rust_solvescramble(s: &str) -> String;
        fn rust_solveposition(s: &str) -> String;
        fn rust_reset();
    }
}

extern crate rouille;

extern crate cubing;
use std::thread::sleep;
use std::time::Duration;

// use cubing::alg::Alg;
use cubing::kpuzzle::KPuzzleDefinition;

use cubing::kpuzzle::KStateData;

// use ffi::w_solvescramble;
use rouille::router;
use rouille::try_or_400;
use rouille::Request;
use rouille::Response;
extern crate serde;
use serde::Deserialize;
use serde::Serialize;
use serialize::serialize_kpuzzle_definition;
use serialize::serialize_scramble_state_data;
use serialize::KPuzzleSerializationOptions;

mod serialize;

fn cors(response: Response) -> Response {
    response
        .with_additional_header("Access-Control-Allow-Origin", "*")
        .with_additional_header("Access-Control-Allow-Headers", "Content-Type")
}

fn set_arg(request: &Request) -> Response {
    let arg = try_or_400!(rouille::input::plain_text_body(request));
    println!("set_arg: {}", arg);
    ffi::rust_arg(&arg); // TODO: catch exceptions???
    Response::empty_204()
}

fn set_definition(
    def: KPuzzleDefinition,
    options: &KPuzzleSerializationOptions,
) -> Result<(), Response> {
    let s = match serialize_kpuzzle_definition(def, Some(options)) {
        Ok(s) => s,
        Err(e) => {
            return Err(Response::text(format!("Invalid definition: {}", e)).with_status_code(400));
        }
    };
    ffi::rust_reset();
    ffi::rust_setksolve(&s); // TODO: catch exceptions???
    sleep(Duration::from_secs(1));
    Err(Response::empty_204())
}

#[derive(Serialize)]
struct ResponseAlg {
    alg: String, // TODO: support automatic alg serialization somehome
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScrambleSolve {
    definition: KPuzzleDefinition,
    scramble_alg: String,
}

// fn solvescramble(request: &Request) -> Response {
//     let scramble_solve: ScrambleSolve = try_or_400!(rouille::input::json_input(request));
//     set_definition(scramble_solve.definition);
//     let alg = match scramble_solve.scramble_alg.parse::<Alg>() {
//         Ok(alg) => alg,
//         Err(_) => return Response::empty_400(), // TODO: more deets
//     };
//     let solution = w_solvescramble(&alg.to_string()); // TODO: catch exceptions???
//     Response::json(&ResponseAlg { alg: solution })
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StateSolve {
    definition: KPuzzleDefinition,
    state: KStateData,
    move_subset: Option<Vec<String>>,
    start_state: Option<KStateData>,
}

fn solveposition(request: &Request) -> Response {
    let state_solve: StateSolve = try_or_400!(rouille::input::json_input(request));
    match set_definition(
        state_solve.definition,
        &KPuzzleSerializationOptions {
            move_subset: state_solve.move_subset,
            custom_start_state: state_solve.start_state,
        },
    ) {
        Ok(_) => {}
        Err(response) => return response,
    };
    let solution = ffi::rust_solveposition(&serialize_scramble_state_data(
        "AnonymousScramble",
        &state_solve.state,
    )); // TODO: catch exceptions???
    Response::json(&ResponseAlg { alg: solution })
}

fn main() {
    // TODO: support parallel requests on the C++ side.
    rouille::start_server("0.0.0.0:2023", /* move */ |request: &Request| {
        println!("Request: {} {}", request.method(), request.url()); // TODO: debug flag
                                                                     // TODO: more fine-grained CORS?
        if request.method() == "OPTIONS" {
            // pre-flight!
            return cors(Response::empty_204());
        }
        cors(router!(request,
            (GET) (/) => {
                Response::text("twsearch-server (https://github.com/cubing/twsearch)")
            },
            (POST) (/v0/config/arg) => {
                set_arg(request)
            },
            // (POST) (/v0/solve/scramble) => {
            //     solvescramble(request)
            // },
            (POST) (/v0/solve/state) => { // TODO: `…/pattern`?
                solveposition(request)
            },
            _ => {
                println!("Invalid request: {} {}", request.method(), request.url());
                rouille::Response::empty_404()
            }
        ))
    });
}
