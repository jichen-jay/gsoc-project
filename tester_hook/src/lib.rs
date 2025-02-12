//the author use this module to test/trigger key project code function by function

use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use gsoc_project::db_join::*;
use gsoc_project::db_manipulate::*;
use gsoc_project::db_populate::*;
use gsoc_project::issue_paced_tracker::*;
use gsoc_project::llm_utils::chat_inner_async;
use gsoc_project::the_paced_runner::*;
use mysql_async::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use vector_store_flows::delete_collection;
use webhook_flows::{
    create_endpoint, request_handler,
    route::{get, post, route, RouteError, Router},
    send_response,
};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler(get, post)]
async fn handler(
    _headers: Vec<(String, String)>,
    _subpath: String,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    dotenv().ok();
    logger::init();

    let mut router = Router::new();
    router.insert("/run", vec![post(trigger)]).unwrap();
    // router
    //     .insert("/deep", vec![post(check_deep_handler)])
    //     .unwrap();
    router
        .insert("/comment", vec![post(get_comments_by_post_handler)])
        .unwrap();

    if let Err(e) = route(router).await {
        match e {
            RouteError::NotFound => {
                send_response(404, vec![], b"No route matched".to_vec());
            }
            RouteError::MethodNotAllowed => {
                send_response(405, vec![], b"Method not allowed".to_vec());
            }
        }
    }
}

async fn get_comments_by_post_handler(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct IssueId {
        pub issue_id: String,
    }

    let load: IssueId = match serde_json::from_slice(&_body) {
        Ok(obj) => obj,
        Err(_e) => {
            log::error!("failed to parse body: {}", _e);
            return;
        }
    };
    let pool: Pool = get_pool().await;

    let issue_id = load.issue_id;
    match get_comments_by_issue_id(&pool, &issue_id).await {
        Ok(result) => {
            let result_str = json!(result).to_string();

            send_response(
                200,
                vec![
                    (
                        String::from("content-type"),
                        String::from("application/json"),
                    ),
                    (
                        String::from("Access-Control-Allow-Origin"),
                        String::from("*"),
                    ),
                ],
                result_str.as_bytes().to_vec(),
            );
        }
        Err(e) => {
            log::error!("Error: {:?}", e);
        }
    }
}
async fn _check_deep_handler(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct VectorLoad {
        pub text: Option<String>,
    }

    if let Ok(load) = serde_json::from_slice::<VectorLoad>(&_body) {
        if let Some(text) = load.text {
            log::info!("text: {text}");
            if let Ok(reply) = chat_inner_async("you're an AI assistant", &text, 100).await {
                send_response(
                    200,
                    vec![
                        (
                            String::from("content-type"),
                            String::from("application/json"),
                        ),
                        (
                            String::from("Access-Control-Allow-Origin"),
                            String::from("*"),
                        ),
                    ],
                    json!(reply).to_string().as_bytes().to_vec(),
                );
            }
        }
    }
}

async fn trigger(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct FuncLoad {
        pub func_ids: Vec<String>,
    }

    let load: FuncLoad = match serde_json::from_slice(&_body) {
        Ok(obj) => obj,
        Err(_e) => {
            log::error!("failed to parse body: {}", _e);
            return;
        }
    };
    let pool: Pool = get_pool().await;
    log::info!("func_id to run: {:?}", load.func_ids);

    for func_id in load.func_ids {
        let _ = match func_id.as_str() {
            "1" => popuate_dbs_save_issues_open(&pool).await,
            "2" => open_master(&pool).await,
            "3" => open_project(&pool).await,
            "4" => popuate_dbs_fill_projects(&pool).await,
            "5" => popuate_dbs_save_issues_closed(&pool).await,
            "6" => closed_master(&pool).await,
            "7" => project_master_back_sync(&pool).await,
            "9" => sum_budget_to_project(&pool).await,
            "10" => popuate_dbs_add_issues_updated(&pool).await,
            "11" => popuate_dbs_save_issues_assign_comment(&pool).await,
            "12" => add_possible_assignees_to_master(&pool).await,
            "13" => popuate_dbs_save_pull_requests(&pool).await,
            "14" => remove_pull_by_issued_linked_pr(&pool).await,
            "15" => delete_issues_open_update_closed(&pool).await,
            // "16" => note_issues(&pool).await, // Uncomment if needed
            _ => panic!("Unhandled function ID: {}", func_id),
        };
    }
}

pub async fn run_hourly(pool: &Pool) -> anyhow::Result<()> {
    // let _ = popuate_dbs(pool).await?;
    // let _ = join_ops(pool).await?;
    // let _ = cleanup_ops(pool).await?;
    Ok(())
}
pub async fn popuate_dbs(pool: &Pool) -> anyhow::Result<()> {
    let query_open =
        "label:hacktoberfest label:hacktoberfest-accepted is:issue closed:2023-10-18..2023-10-20 -label:spam -label:invalid";

    let open_issue_obj: Vec<IssueOpen> = search_issues_open(&query_open).await?;
    let len = open_issue_obj.len();
    log::info!("Open Issues recorded: {:?}", len);
    for issue in open_issue_obj {
        let _ = add_issues_open(pool, &issue).await;

        let _ = summarize_issue_add_in_db(pool, &issue).await;
    }

    // let query_comment =
    //     "label:hacktoberfest label:hacktoberfest-accepted is:issue closed:2023-10-12..2023-10-18 -label:spam -label:invalid";
    // log::info!("query_open: {:?}", query_open);

    // let issue_comment_obj: Vec<IssueComment> = search_issues_comment(&query_comment).await?;
    // let len = issue_comment_obj.len();
    // log::info!("Issues comment recorded: {:?}", len);
    // for issue in issue_comment_obj {
    //     let _ = add_issues_comment(pool, issue).await;
    // }

    // let _query_assigned =
    //     "label:hacktoberfest label:hacktoberfest-accepted is:issue closed:2023-10-12..2023-10-18 -label:spam -label:invalid";
    // let issues_assigned_obj: Vec<IssueAssigned> = search_issues_assigned(&_query_assigned).await?;
    // let len = issues_assigned_obj.len();
    // log::info!("Assigned issues recorded: {:?}", len);
    // for issue in issues_assigned_obj {
    //     let _ = add_issues_assigned(pool, issue).await;
    // }

    let query_closed =
        "label:hacktoberfest label:hacktoberfest-accepted is:issue closed:2023-10-18..2023-10-20 -label:spam -label:invalid";
    let close_issue_obj = search_issues_closed(&query_closed).await?;
    let len = close_issue_obj.len();
    log::info!("Closed issues recorded: {:?}", len);
    for issue in close_issue_obj {
        let _ = add_issues_closed(pool, issue).await;
    }

    Ok(())
}


pub async fn join_ops(pool: &Pool) -> anyhow::Result<()> {
    let _ = open_master(&pool).await?;
    // let _ = assigned_master(&pool).await?;

    let _ = closed_master(&pool).await?;

    let _ = add_possible_assignees_to_master(&pool).await?;
    // let _ = sum_budget_to_project(&pool).await?;

    let query_repos: String = get_projects_as_repo_list(pool, 1).await?;
    log::info!("repos list: {:?}", query_repos.clone());

    let repo_data_vec: Vec<RepoData> = search_repos_in_batch(&query_repos).await?;

    for repo_data in repo_data_vec {
        log::info!("repo : {:?}", repo_data.project_id.clone());

        let _ = fill_project_w_repo_data(&pool, repo_data.clone()).await?;
        let _ = summarize_project_add_in_db(&pool, repo_data).await?;
    }

    Ok(())
}

pub async fn cleanup_ops(pool: &Pool) -> anyhow::Result<()> {
    let _ = remove_pull_by_issued_linked_pr(&pool).await?;
    let _ = delete_issues_open_update_closed(&pool).await?;

    Ok(())
}
