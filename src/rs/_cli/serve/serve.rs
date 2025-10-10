use cubing::kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition};

use rouille::{router, try_or_400, Request, Response};
use serde::{Deserialize, Serialize};
use twsearch::_internal::{
    canonical_fsm::search_generators::SearchGenerators,
    cli::args::{
        CustomGenerators, Generators, MetricEnum, ServeArgsForIndividualSearch, ServeClientArgs,
        ServeCommandArgs,
    },
    errors::CommandError,
    search::{
        iterative_deepening::{
            individual_search::IndividualSearchOptions,
            iterative_deepening_search::{
                ImmutableSearchData, ImmutableSearchDataConstructionOptions,
                IterativeDeepeningSearch,
            },
            search_adaptations::StoredSearchAdaptations,
        },
        search_logger::SearchLogger,
    },
};

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Serialize)]
struct ResponseAlg {
    alg: String, // TODO: support automatic alg serialization somehome
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KPatternSolve {
    definition: KPuzzleDefinition,
    pattern: KPatternData,
    start_pattern: Option<KPatternData>,
    search_args: Option<ServeClientArgs>,
}

fn solve_pattern(
    request: &Request,
    serve_command_args: &ServeCommandArgs,
    request_counter: usize,
) -> Response {
    println!("[Search request #{}] Starting search…", request_counter);
    let start_time = instant::Instant::now();
    let kpattern_solve: KPatternSolve = try_or_400!(rouille::input::json_input(request));
    // TODO: use the client args
    let args_for_individual_search = ServeArgsForIndividualSearch {
        commandline_args: serve_command_args,
        client_args: &kpattern_solve.search_args,
    };
    let kpuzzle = match KPuzzle::try_new(kpattern_solve.definition) {
        Ok(kpuzzle) => kpuzzle.clone(),
        Err(e) => return Response::text(e.description).with_status_code(400),
    };
    let target_pattern = match kpattern_solve.start_pattern {
        Some(kpattern_data) => match KPattern::try_from_data(&kpuzzle, &kpattern_data) {
            Ok(target_pattern) => target_pattern,
            Err(e) => return Response::text(e.to_string()).with_status_code(400),
        },
        None => kpuzzle.default_pattern(),
    };
    let search_logger = Arc::new(SearchLogger {
        verbosity: args_for_individual_search
            .commandline_args
            .verbosity_args
            .verbosity
            .unwrap_or_default(),
    });
    let move_subset = match args_for_individual_search.client_args {
        Some(client_args) => client_args.generator_moves.as_ref().cloned(),
        None => None,
    };
    let move_list =
        move_subset.unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect());
    let search_pattern = match KPattern::try_from_data(&kpuzzle, &kpattern_solve.pattern) {
        Ok(search_pattern) => search_pattern,
        Err(e) => return Response::text(e.to_string()).with_status_code(400),
    };
    let search_generators = match SearchGenerators::try_new(
        &kpuzzle,
        Generators::Custom(CustomGenerators {
            moves: move_list.clone(),
            algs: vec![],
        })
        .enumerate_moves_for_kpuzzle(&kpuzzle),
        match args_for_individual_search.client_args {
            Some(client_args) => {
                if client_args.quantum_metric.unwrap_or_default() {
                    &MetricEnum::Quantum
                } else {
                    &MetricEnum::Hand
                }
            }
            None => &MetricEnum::Hand,
        },
        match args_for_individual_search.client_args {
            Some(client_args) => client_args.random_start == Some(true),
            None => false,
        },
    ) {
        Ok(search_generators) => search_generators,
        Err(e) => return Response::text(e.description).with_status_code(400),
    };
    let immutable_search_data = match ImmutableSearchData::try_from_common_options(
        kpuzzle.clone(),
        search_generators,
        vec![target_pattern], // TODO: modify api to support multiple target patterns
        ImmutableSearchDataConstructionOptions {
            search_logger,
            ..Default::default()
        },
    ) {
        Ok(immutable_search_data) => immutable_search_data,
        Err(e) => return Response::text(e.description).with_status_code(400),
    };
    let mut search = <IterativeDeepeningSearch<KPuzzle>>::new_with_hash_prune_table(
        immutable_search_data,
        StoredSearchAdaptations::default(),
        Default::default(),
        // IterativeDeepeningSearchConstructionOptions {
        //     random_start: match args_for_individual_search.client_args {
        //         Some(client_args) => client_args.random_start == Some(true),
        //         None => false,
        //     },
        //     ..Default::default()
        // },
        // None,
    );
    if let Some(solution) = search
        .search(
            &search_pattern,
            IndividualSearchOptions {
                min_num_solutions: None,
                min_depth_inclusive: args_for_individual_search
                    .client_args
                    .as_ref()
                    .and_then(|client_args| client_args.min_depth),
                max_depth_exclusive: args_for_individual_search
                    .client_args
                    .as_ref()
                    .and_then(|client_args| client_args.max_depth),
                // TODO: support canonical FSM pre-moves and post-moves.
                ..Default::default()
            },
            Default::default(),
        )
        .next()
    {
        println!(
            "[Search request #{}] Solution found (in {:?}): {}",
            request_counter,
            instant::Instant::now() - start_time,
            solution
        );
        return Response::json(&ResponseAlg {
            alg: solution.to_string(),
        }); // TODO: send multiple solutions via socket
    }
    println!("[Search request #{}] No solution found.", request_counter);
    Response::text("No solution found").with_status_code(404)
}

fn cors(response: Response) -> Response {
    response
        .with_additional_header("Access-Control-Allow-Origin", "*")
        .with_additional_header("Access-Control-Allow-Headers", "Content-Type")
}

pub fn serve(serve_command_args: ServeCommandArgs) -> Result<(), CommandError> {
    let search_request_counter = Arc::new(Mutex::<usize>::new(0));
    println!(
        "Starting `twsearch serve` on port 2023.
Use with one of the following:

- https://experiments.cubing.net/cubing.js/twsearch/text-ui.html
- http://localhost:3333/experiments.cubing.net/cubing.js/twsearch/text-ui.html
"
    );
    rouille::start_server("0.0.0.0:2023", move |request: &Request| {
        println!("Request: {} {}", request.method(), request.url()); // TODO: debug flag
                                                                     // TODO: more fine-grained CORS?
        if request.method() == "OPTIONS" {
            // pre-flight!
            return cors(Response::empty_204());
        }
        cors(router!(request,
            (GET) (/) => {
                Response::text("twsearch-cpp-wrapper (https://github.com/cubing/twsearch)")
            },
            (POST) (/v0/solve/pattern) => { // TODO: `…/pattern`?
                let mut counter = search_request_counter
                    .lock()
                    .expect("Internal error: could not access request counter");
                *counter += 1;
                let local_counter = *counter;
                drop(counter);
                solve_pattern(request, &serve_command_args, local_counter)
            },
            _ => {
                println!("Invalid request: {} {}", request.method(), request.url());
                rouille::Response::empty_404()
            }
        ))
    });
}
