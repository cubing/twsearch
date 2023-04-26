extern crate cxx;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("twsearch/src/cpp/wasmapi.h");
        fn w_arg(s: &str);
        fn w_setksolve(s: &str);
        fn w_solvescramble(s: &str) -> String;
        fn w_solveposition(s: &str) -> String;
    }
}

extern crate rouille;

extern crate cubing;
use std::thread::sleep;
use std::time::Duration;

use cubing::alg::Alg;
use cubing::kpuzzle::KPuzzleDefinition;

use cubing::kpuzzle::KStateData;

use ffi::w_setksolve;
use ffi::w_solveposition;
use ffi::w_solvescramble;
use rouille::router;
use rouille::try_or_400;
use rouille::Request;
use rouille::Response;
extern crate serde;
use serde::Serialize;
use serialize::serialize_kpuzzle_definition;
use serialize::serialize_kstate_data;

mod serialize;

use crate::ffi::w_arg;

fn cors(response: Response) -> Response {
    response
        .with_additional_header("Access-Control-Allow-Origin", "*")
        .with_additional_header("Access-Control-Allow-Headers", "Content-Type")
}

fn set_arg(request: &Request) -> Response {
    let arg = try_or_400!(rouille::input::plain_text_body(request));
    println!("set_arg: {}", arg);
    w_arg(&arg); // TODO: catch exceptions???
    Response::empty_204()
}

fn set_ksolve(request: &Request) -> Response {
    let def: KPuzzleDefinition = try_or_400!(rouille::input::json_input(request));
    w_setksolve(serialize_kpuzzle_definition(&def)); // TODO: catch exceptions???
    sleep(Duration::from_secs(1));
    Response::empty_204()
}

#[derive(Serialize)]
struct ResponseAlg {
    alg: String, // TODO: support automatic alg serialization somehome
}

fn solvescramble(request: &Request) -> Response {
    let arg = try_or_400!(rouille::input::plain_text_body(request));
    let alg = match arg.parse::<Alg>() {
        Ok(alg) => alg,
        Err(_) => return Response::empty_400(), // TODO: more deets
    };
    let solution = w_solvescramble(&alg.to_string()); // TODO: catch exceptions???
    Response::json(&ResponseAlg { alg: solution })
}

fn solveposition(request: &Request) -> Response {
    let kstate_data: KStateData = try_or_400!(rouille::input::json_input(request));
    let solution = w_solveposition(serialize_kstate_data(&kstate_data)); // TODO: catch exceptions???
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
            (POST) (/v0/config/definition) => {
                set_ksolve(request)
            },
            (POST) (/v0/solve/scramble) => {
                solvescramble(request)
            },
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
