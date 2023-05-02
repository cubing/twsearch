extern crate cxx;

extern crate lazy_static;
extern crate regex;
use std::sync::Mutex;

extern crate cubing;
use cubing::kpuzzle::KPuzzleDefinition;
use cubing::kpuzzle::KStateData;

extern crate rouille;
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
    ffi::rust_setksolve(&s);
    Ok(())
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
    let solve_mutex = Mutex::new(());
    rouille::start_server("0.0.0.0:2023", move |request: &Request| {
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
            (POST) (/v0/solve/state) => { // TODO: `…/pattern`?
                if let Ok(guard) = solve_mutex.try_lock() {
                    let response = solveposition(request);
                    drop(guard);
                    response
                } else {
                    // TODO: support aborting the search on the C++ side.
                    Response::text("Only one non-abortable search at a time is currently supported.").with_status_code(409)
                }
            },
            _ => {
                println!("Invalid request: {} {}", request.method(), request.url());
                rouille::Response::empty_404()
            }
        ))
    });
}
