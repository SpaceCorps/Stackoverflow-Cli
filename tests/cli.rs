//! Drives the built binary against an in-process mock of the Apify API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-without-leading-slash, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let routes = Arc::new(Mutex::new(routes));
        let log2 = log.clone();
        let routes2 = routes.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes2.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let full_path = parts.next().unwrap_or("").trim_start_matches('/').to_string();
                    let path_only = full_path.split('?').next().unwrap_or("").to_string();

                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap());
                    log.lock().unwrap().push(Recorded {
                        method: method.clone(),
                        path: full_path.clone(),
                        headers,
                        body,
                    });

                    let (status, resp) = {
                        let mut r = routes.lock().unwrap();
                        let matches: Vec<usize> = r
                            .iter()
                            .enumerate()
                            .filter(|(_, (m, p, _, _))| *m == method && (*p == path_only || *p == full_path))
                            .map(|(i, _)| i)
                            .collect();
                        if matches.is_empty() {
                            (404, json!({"error": {"message": "not found"}}))
                        } else if matches.len() > 1 {
                            let idx = matches[0];
                            let (_, _, s, b) = r.remove(idx);
                            (s, b)
                        } else {
                            let idx = matches[0];
                            let (_, _, s, ref b) = r[idx];
                            (s, b.clone())
                        }
                    };

                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "stackoverflow-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_stackoverflow"))
            .args(args)
            .env("STACKOVERFLOW_CONFIG_DIR", &self.dir)
            .env("STACKOVERFLOW_SECRET_STORE", "plaintext")
            .env("APIFY_API_URL", &self.api)
            .env_remove("APIFY_TOKEN")
            .output()
            .unwrap()
    }

    fn run_with_env(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_stackoverflow"));
        cmd.args(args)
            .env("STACKOVERFLOW_CONFIG_DIR", &self.dir)
            .env("STACKOVERFLOW_SECRET_STORE", "plaintext")
            .env("APIFY_API_URL", &self.api)
            .env_remove("APIFY_TOKEN");
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn json_with_env(&self, args: &[&str], envs: &[(&str, &str)]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run_with_env(&all, envs);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    /// Adds account `work` with key `apify_test`.
    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "apify_test"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn user_verify_route() -> Route {
    (
        "GET",
        "users/me",
        200,
        json!({
            "data": {
                "id": "user-123",
                "username": "spacecorps",
                "email": "agent@spacecorps.io"
            }
        }),
    )
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![user_verify_route()]);
    let env = Env::new(&mock).with_account();

    let last_req = mock.last("GET");
    assert!(last_req.path.starts_with("users/me"));
    assert_eq!(last_req.headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer apify_test");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["name"], "work");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");
    assert_eq!(out["secretStore"], "plaintext");

    let (_, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    // Duplicate account name requires --force
    let (code, _, err) = env.json(&["accounts", "add", "WORK", "--api-key", "x"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    let (code, out, _) = env.json(&["accounts", "test", "Work"]);
    assert_eq!(code, 0);
    assert_eq!(out["keyStatus"], "valid");

    // No terminal and no --yes refuses
    let (code, _, err) = env.json(&["accounts", "remove", "work"]);
    assert_eq!(code, 6);
    assert_eq!(err["remediation"], "stackoverflow accounts remove work --yes");

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (_, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(out["count"], 0);
}

#[test]
fn api_key_from_stdin() {
    let mock = Mock::start(vec![user_verify_route()]);
    let env = Env::new(&mock);
    let mut child = Command::new(env!("CARGO_BIN_EXE_stackoverflow"))
        .args(["accounts", "add", "piped", "--api-key-stdin", "--json"])
        .env("STACKOVERFLOW_CONFIG_DIR", &env.dir)
        .env("STACKOVERFLOW_SECRET_STORE", "plaintext")
        .env("APIFY_API_URL", &env.api)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"apify_piped_token\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let auth = mock.last("GET").headers.into_iter().find(|(k, _)| k == "authorization").unwrap().1;
    assert_eq!(auth, "Bearer apify_piped_token");
}

#[test]
fn config_is_readable_yaml_without_secrets() {
    let mock = Mock::start(vec![user_verify_route()]);
    let env = Env::new(&mock).with_account();
    let yaml = std::fs::read_to_string(env.dir.join("config.yaml")).unwrap();
    assert!(yaml.contains("work:"), "{yaml}");
    assert!(!yaml.contains("apify_test"), "the key leaked into config.yaml");
}

#[test]
fn search_command_payload() {
    let mock = Mock::start(vec![
        user_verify_route(),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": 12345678,
                    "title": "How to optimize Rust memory usage?",
                    "url": "https://stackoverflow.com/questions/12345678/how-to-optimize-rust-memory-usage",
                    "score": 42,
                    "tags": ["rust", "memory"]
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "search",
        "Rust memory optimization",
        "--account",
        "work",
        "--tagged",
        "rust,memory",
        "--answers",
        "--site",
        "stackoverflow",
        "--max",
        "15",
        "--sort",
        "votes",
    ]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["title"], "How to optimize Rust memory usage?");

    let last_req = mock.last("POST");
    assert!(last_req.path.contains("acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items"));
    assert!(last_req.path.contains("token=apify_test"));

    let body = last_req.body.unwrap();
    assert_eq!(body["keywords"], "Rust memory optimization");
    assert_eq!(body["tags"], json!(["rust", "memory"]));
    assert_eq!(body["site"], "stackoverflow");
    assert_eq!(body["includeAnswers"], true);
    assert_eq!(body["mode"], "search");
    assert_eq!(body["maxResults"], 15);
    assert_eq!(body["sort"], "votes");
}

#[test]
fn scrape_command_tags_payload() {
    let mock = Mock::start(vec![
        user_verify_route(),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": 87654321,
                    "title": "Autonomous agent design in Rust",
                    "tags": ["ai-agent", "llm"]
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) =
        env.json(&["scrape", "ai-agent,llm", "--account", "work", "--max", "20", "--sort", "newest"]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["title"], "Autonomous agent design in Rust");

    let last_req = mock.last("POST");
    let body = last_req.body.unwrap();
    assert_eq!(body["tags"], json!(["ai-agent", "llm"]));
    assert_eq!(body["includeAnswers"], true);
    assert_eq!(body["mode"], "tags");
    assert_eq!(body["maxResults"], 20);
    assert_eq!(body["sort"], "newest");
}

#[test]
fn scrape_command_tagged_url_payload() {
    let mock = Mock::start(vec![
        user_verify_route(),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": 999,
                    "title": "WebAssembly in Rust",
                    "tags": ["rust", "wasm"]
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "scrape",
        "https://stackoverflow.com/questions/tagged/rust+wasm?tab=newest",
        "--account",
        "work",
        "--no-answers",
    ]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["title"], "WebAssembly in Rust");

    let last_req = mock.last("POST");
    let body = last_req.body.unwrap();
    assert_eq!(body["tags"], json!(["rust", "wasm"]));
    assert_eq!(body["includeAnswers"], false);
    assert_eq!(body["mode"], "tags");
}

#[test]
fn scrape_command_question_url_payload() {
    let mock = Mock::start(vec![
        user_verify_route(),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": 12345,
                    "title": "Understanding lifetimes",
                    "body": "How do lifetimes work?"
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) =
        env.json(&["scrape", "https://stackoverflow.com/questions/12345/understanding-lifetimes", "--account", "work"]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["title"], "Understanding lifetimes");

    let last_req = mock.last("POST");
    let body = last_req.body.unwrap();
    assert_eq!(body["keywords"], "understanding lifetimes");
    assert_eq!(body["mode"], "search");
}

#[test]
fn direct_api_key_and_env_var() {
    let mock = Mock::start(vec![
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([{"title": "Direct Key Search"}]),
        ),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([{"title": "Env Key Search"}]),
        ),
    ]);
    let env = Env::new(&mock);

    // 1. Direct --api-key
    let (code, out, err) = env.json(&["search", "direct test", "--api-key", "token_direct"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["title"], "Direct Key Search");
    assert!(mock.last("POST").path.contains("token=token_direct"));

    // 2. APIFY_TOKEN environment variable
    let (code, out, err) = env.json_with_env(&["search", "env test"], &[("APIFY_TOKEN", "token_from_env")]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["title"], "Env Key Search");
    assert!(mock.last("POST").path.contains("token=token_from_env"));
}

#[test]
fn auth_is_required_when_unconfigured() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["search", "anything"]);
    assert_eq!(code, 7);
    assert_eq!(err["code"], "no_account");
    assert!(err["remediation"].as_str().unwrap().contains("stackoverflow login"));
}

#[test]
fn http_errors_map_to_exit_codes() {
    let mock = Mock::start(vec![
        user_verify_route(),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            401,
            json!({"error": {"message": "Invalid token"}}),
        ),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            403,
            json!({"error": {"message": "Forbidden actor"}}),
        ),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            404,
            json!({"error": {"message": "Actor not found"}}),
        ),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            429,
            json!({"error": {"message": "Rate limit exceeded"}}),
        ),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            500,
            json!({"error": {"message": "Internal Apify error"}}),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    // 401 -> 3 (AuthRequired)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    // 403 -> 3 (AuthRequired)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    // 404 -> 4 (NotFound)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 4);
    assert_eq!(err["code"], "not_found");

    // 429 -> 5 (RateLimited)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");

    // 500 -> 2 (Network)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 2);
    assert_eq!(err["code"], "network");
}

#[test]
fn parse_errors_are_envelopes() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    // Missing positional query argument
    let (code, _, err) = env.json(&["search"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
    assert!(err["error"].as_str().unwrap().contains("<QUERY>"));

    // Missing positional target argument for scrape
    let (code, _, err) = env.json(&["scrape"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
    assert!(err["error"].as_str().unwrap().contains("<TARGET>"));

    // --help exits successfully
    let out = env.run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn yaml_is_the_default() {
    let mock = Mock::start(vec![
        user_verify_route(),
        (
            "POST",
            "acts/sheshinmcfly~stackoverflow-scraper/run-sync-get-dataset-items",
            200,
            json!([{"title": "YAML Output Result", "url": "https://stackoverflow.com/questions/1"}]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let out = env.run(&["search", "yaml test", "-a", "work"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("title: YAML Output Result"), "{stdout}");
    assert!(stdout.contains("url: https://stackoverflow.com/questions/1"), "{stdout}");
}

#[test]
fn agent_readme_as_data() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "stackoverflow");
    assert_eq!(out["exitCodes"]["7"], "no_account - run stackoverflow accounts list or set APIFY_TOKEN");

    let md = String::from_utf8(env.run(&["agent-readme"]).stdout).unwrap();
    assert!(md.starts_with("# stackoverflow - agent operating manual"));
}

#[test]
fn login_command_lifecycle() {
    let mock = Mock::start(vec![user_verify_route(), user_verify_route(), user_verify_route()]);
    let env = Env::new(&mock);

    // Initial login
    let (code, out, err) = env.json(&["login", "--api-key", "apify_login_token"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "logged_in");
    assert_eq!(out["name"], "default");
    assert_eq!(out["secretStore"], "plaintext");

    // Login without --force refuses duplicate
    let (code, _, err) = env.json(&["login", "--api-key", "apify_login_token_2"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    // Login with --force succeeds
    let (code, out, _) = env.json(&["login", "--api-key", "apify_login_token_2", "--force"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "logged_in");

    // Login with named account
    let (code, out, _) = env.json(&["login", "production", "--api-key", "apify_prod_token"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "production");
}
