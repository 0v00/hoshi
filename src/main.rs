use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use futures;
use serde_json::Value;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let user_with_stars: String = env::var("USER_WITH_STARS").expect("USER_WITH_STARS val not found");
    let my_gh_token: String = env::var("GH_TOKEN").expect("GH_TOKEN val not found");

    let request_url: String = format!("https://api.github.com/users/{}/starred?per_page=100", user_with_stars);

    let client: Client = Client::new();
    let client = Arc::new(client);

    let starred_repos_response: reqwest::Response = client
        .get(&request_url)
        .header("User-Agent", "0v00")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", my_gh_token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send().await?;

    let body: String = starred_repos_response.text().await?;
    let json: Value = serde_json::from_str(&body)?;

    // old approach to repos_to_star
    // let mut repos_to_star: Vec<String> = Vec::new();
    // if let Some(starred_repos) = json.as_array() {
    //     for repo in starred_repos {
    //         let full_name: &str = repo["full_name"].as_str().unwrap();
    //         let repo_to_star = format!("https://api.github.com/user/{}", full_name);
    //         repos_to_star.push(repo_to_star);
    //     }
    // }

    let repos_to_star: Vec<String> = json
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|repo| repo["full_name"].as_str())
        .map(|full_name| format!("https://api.github.com/user/starred/{}", full_name))
        .collect();

    for repo in repos_to_star.chunks(2) {
        let tasks: Vec<_> = repo
            .iter()
            .map(|url| {
                let client = Arc::clone(&client);
                let token = my_gh_token.clone();
                let url_clone = url.clone();
                tokio::spawn(async move {
                    let response = client
                        .put(&url_clone)
                        .header("User-Agent", "0v00")
                        .header("Accept", "application/vnd.github+json")
                        .header("Authorization", format!("Bearer {}", token))
                        .header("X-GitHub-Api-Version", "2022-11-28")
                        .send().await;

                    match response {
                        Ok(response) => {
                            if response.status() == 204 {
                                println!("Successfully starred repo: {}", url_clone);
                            } else {
                                eprintln!("Failed to star repo: {}", url_clone);
                            }
                        },
                        Err(e) => eprintln!("Error sending request: {}", e),
                    }
                })
            })
            .collect();

        let _results: Vec<_> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .collect();
    }

    Ok(())
}