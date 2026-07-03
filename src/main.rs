use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MODEL: &str = "@makers/deepseek-v4-flash";
const DEFAULT_ENDPOINT: &str = "https://ai-gateway.edgeone.link/v1";
const PROMPT: &str = "You are Neomux, a token-efficient coding agent running beside Neovim in tmux.\n\nRules:\n- Be concise and execution-focused.\n- Prefer minimal, explicit code over clever abstractions.\n- Do not chase theoretical completeness.\n- Avoid speculative edge-case handling.\n- Do not add defensive code unless it prevents a realistic failure.\n- Prefer small diffs and direct commands.\n- Ask before destructive actions.\n- When commands are needed, give exact commands.";

const RESET: &str = "\x1b[0m";
const LOVE: &str = "\x1b[38;2;235;111;146m";
const IRIS: &str = "\x1b[38;2;196;167;231m";
const MUTED: &str = "\x1b[38;2;110;106;134m";
const GOLD: &str = "\x1b[38;2;246;193;119m";

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("agent") => run_agent(),
        Some("doctor") => doctor(),
        Some("models") => models(),
        Some("config") => config(),
        Some("version") => version(args.get(1).is_some_and(|arg| arg == "--json")),
        Some("--version") | Some("-V") => version(false),
        Some("--help") | Some("-h") => help(),
        _ => launch_tmux(&args),
    }
}

fn help() -> Result<(), String> {
    println!("neomux [path] [--session <name>] [--no-attach]");
    println!();
    println!("Commands:");
    println!("  neomux doctor          check local setup");
    println!("  neomux models          list bundled model metadata");
    println!("  neomux config          show active config");
    println!("  neomux version --json  print machine-readable version info");
    println!("  neomux agent           run agent pane directly");
    Ok(())
}

fn launch_tmux(args: &[String]) -> Result<(), String> {
    let mut target = env::current_dir().map_err(|error| error.to_string())?;
    let mut session = env::var("NEOMUX_SESSION").ok();
    let mut attach = true;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--no-attach" => attach = false,
            "--session" => {
                index += 1;
                session = Some(args.get(index).ok_or("--session requires a name")?.clone());
            }
            "-h" | "--help" => return help(),
            value if !value.starts_with('-') => target = PathBuf::from(value),
            value => return Err(format!("unknown argument: {value}")),
        }
        index += 1;
    }

    require_command("tmux")?;
    require_command("nvim")?;
    let cwd = target.canonicalize().unwrap_or(target);
    let name = session.unwrap_or_else(|| {
        let base = cwd
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace");
        format!("neomux-{}", slug(base))
    });

    if Command::new("tmux")
        .args(["has-session", "-t", &name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
    {
        if attach {
            tmux(["attach-session", "-t", &name])?;
        }
        println!("neomux session already running: {name}");
        return Ok(());
    }

    let editor = tmux_output([
        "new-session",
        "-d",
        "-P",
        "-F",
        "#{pane_id}",
        "-s",
        &name,
        "-c",
        path_str(&cwd)?,
    ])?;
    tmux(["rename-window", "-t", &editor, "neomux"])?;
    tmux([
        "set-option",
        "-t",
        &name,
        "status-style",
        "bg=#191724,fg=#e0def4",
    ])?;
    tmux([
        "set-option",
        "-t",
        &name,
        "status-left",
        "#[fg=#eb6f92,bold] neomux #[fg=#c4a7e7]",
    ])?;
    tmux([
        "set-option",
        "-t",
        &name,
        "pane-active-border-style",
        "fg=#eb6f92",
    ])?;
    tmux(["set-option", "-t", &name, "pane-border-style", "fg=#6e6a86"])?;
    tmux([
        "send-keys",
        "-t",
        &editor,
        "nvim '+silent! colorscheme rose-pine' .",
        "C-m",
    ])?;

    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let agent_command = format!("{} agent", shell_quote(path_str(&exe)?));
    let agent = tmux_output([
        "split-window",
        "-P",
        "-F",
        "#{pane_id}",
        "-h",
        "-t",
        &editor,
        "-c",
        path_str(&cwd)?,
    ])?;
    tmux(["send-keys", "-t", &agent, &agent_command, "C-m"])?;
    tmux(["select-pane", "-t", &editor])?;

    if attach {
        tmux(["attach-session", "-t", &name])?;
    } else {
        println!("neomux session started: {name}");
    }
    Ok(())
}

fn run_agent() -> Result<(), String> {
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let mut state = AgentState::new();
    println!(
        "{LOVE}neomux{RESET} {IRIS}agent{RESET}  {MUTED}cwd:{RESET} {}",
        cwd.display()
    );
    println!("{MUTED}model:{RESET} {}", state.model);
    println!("{MUTED}commands:{RESET} /help /find /read /context /run /model /clear /exit");

    let stdin = io::stdin();
    loop {
        print!("{LOVE}neomux>{RESET} ");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut line = String::new();
        stdin
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "/exit" {
            break;
        }
        if let Err(error) = handle_agent_line(line, &cwd, &mut state) {
            println!("{LOVE}{error}{RESET}");
        }
    }
    Ok(())
}

fn handle_agent_line(line: &str, cwd: &Path, state: &mut AgentState) -> Result<(), String> {
    if !line.starts_with('/') {
        return chat(line, state);
    }
    let (command, arg) = split_command(line);
    match command {
        "/help" => agent_help(),
        "/clear" => {
            state.conversation.clear();
            println!("conversation cleared");
            Ok(())
        }
        "/context" => {
            println!(
                "{MUTED}files:{RESET} {}",
                if state.files.is_empty() {
                    "none".into()
                } else {
                    state.files.len().to_string()
                }
            );
            for file in &state.files {
                println!("  {} ({} chars)", file.path, file.text.len());
            }
            println!(
                "{MUTED}recent commands:{RESET} {}",
                if state.commands.is_empty() {
                    "none".into()
                } else {
                    state.commands.len().to_string()
                }
            );
            for command in &state.commands {
                println!(
                    "  {}: exit {} in {}ms",
                    command.label, command.code, command.ms
                );
            }
            Ok(())
        }
        "/find" => {
            let output = run_capture(
                "rg",
                &[
                    "--line-number",
                    "--column",
                    "--color",
                    "never",
                    "--fixed-strings",
                    arg,
                    ".",
                ],
                cwd,
            )?;
            print!("{}", output.text);
            println!("{MUTED}exit:{RESET} {} in {}ms", output.code, output.ms);
            state.push_command(format!("/find {arg}"), output);
            Ok(())
        }
        "/read" => {
            if arg.is_empty() {
                return Err("usage: /read <file>".into());
            }
            let path = cwd
                .join(arg)
                .canonicalize()
                .map_err(|error| error.to_string())?;
            if !path.starts_with(cwd) {
                return Err("path must stay inside the workspace".into());
            }
            let mut text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            if text.len() > 50_000 {
                text.truncate(50_000);
            }
            state.files.retain(|file| file.path != arg);
            state.files.push(LoadedFile {
                path: arg.to_string(),
                text,
            });
            println!("loaded {arg}");
            Ok(())
        }
        "/forget" => {
            if arg.is_empty() || arg == "all" {
                let count = state.files.len();
                state.files.clear();
                println!("removed {count} context item(s)");
            } else {
                let before = state.files.len();
                state.files.retain(|file| file.path != arg);
                println!("removed {} context item(s)", before - state.files.len());
            }
            Ok(())
        }
        "/run" => {
            if arg.is_empty() {
                return Err("usage: /run <cmd>".into());
            }
            if let Some(reason) = risk(arg) {
                print!("{GOLD}risk: {reason}. Type \"run\" to continue:{RESET} ");
                io::stdout().flush().map_err(|error| error.to_string())?;
                let mut answer = String::new();
                io::stdin()
                    .read_line(&mut answer)
                    .map_err(|error| error.to_string())?;
                if answer.trim() != "run" {
                    println!("command cancelled");
                    return Ok(());
                }
            }
            println!("{MUTED}cwd:{RESET} {}", cwd.display());
            let output = run_shell(arg, cwd)?;
            print!("{}", output.text);
            if !output.text.ends_with('\n') && !output.text.is_empty() {
                println!();
            }
            println!("{MUTED}exit:{RESET} {} in {}ms", output.code, output.ms);
            state.push_command(arg.to_string(), output);
            Ok(())
        }
        "/model" => {
            if arg.is_empty() {
                return Err("usage: /model <id>".into());
            }
            state.model = arg.to_string();
            println!("{MUTED}model:{RESET} {}", state.model);
            Ok(())
        }
        "/temperature" => {
            state.temperature = Some(
                arg.parse::<f32>()
                    .map_err(|_| "usage: /temperature <number>".to_string())?,
            );
            println!("{MUTED}temperature:{RESET} {}", arg);
            Ok(())
        }
        "/max-tokens" => {
            state.max_tokens = Some(
                arg.parse::<u32>()
                    .map_err(|_| "usage: /max-tokens <integer>".to_string())?,
            );
            println!("{MUTED}max tokens:{RESET} {}", arg);
            Ok(())
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn agent_help() -> Result<(), String> {
    println!("/find <text>          search workspace");
    println!("/read <file>          load file context");
    println!("/context              show loaded context");
    println!("/forget [file|all]    remove context");
    println!("/run <cmd>            run shell command");
    println!("/model <id>           switch model");
    println!("/temperature <n>      set temperature");
    println!("/max-tokens <n>       set max output tokens");
    println!("/clear                clear chat history");
    println!("/exit                 quit");
    Ok(())
}

fn chat(prompt: &str, state: &mut AgentState) -> Result<(), String> {
    let key = env::var("MAKERS_MODELS_KEY").map_err(|_| {
        "missing MAKERS_MODELS_KEY. Fix: export MAKERS_MODELS_KEY=\"...\"".to_string()
    })?;
    let started = Instant::now();
    let body = state.chat_body(prompt);
    let endpoint = format!("{}/chat/completions", state.endpoint.trim_end_matches('/'));
    let mut child = Command::new("curl")
        .args(["-sS", "-N", "-X", "POST", &endpoint])
        .args(["-H", &format!("Authorization: Bearer {key}")])
        .args(["-H", "Content-Type: application/json"])
        .args(["--data", &body])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| {
            "curl is required for EdgeOne chat. Fix: install curl or use the hosted proxy later."
                .to_string()
        })?;

    let stdout = child.stdout.take().ok_or("failed to read curl output")?;
    let reader = BufReader::new(stdout);
    let mut answer = String::new();
    for line in reader.lines() {
        let line = line.map_err(|error| error.to_string())?;
        if let Some(token) = sse_content(&line) {
            print!("{token}");
            io::stdout().flush().map_err(|error| error.to_string())?;
            answer.push_str(&token);
        }
    }
    let status = child.wait().map_err(|error| error.to_string())?;
    println!();
    println!(
        "{MUTED}[{} | {} chars | {}ms]{RESET}",
        state.model,
        answer.len(),
        started.elapsed().as_millis()
    );
    if !status.success() {
        return Err(format!("EdgeOne request failed with curl status {status}"));
    }
    state.conversation.push(("user".into(), prompt.into()));
    state.conversation.push(("assistant".into(), answer));
    while state.conversation.len() > 30 {
        state.conversation.remove(0);
    }
    Ok(())
}

struct AgentState {
    endpoint: String,
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    files: Vec<LoadedFile>,
    commands: VecDeque<CommandResult>,
    conversation: Vec<(String, String)>,
}

impl AgentState {
    fn new() -> Self {
        Self {
            endpoint: env::var("EDGEONE_BASE_URL").unwrap_or_else(|_| DEFAULT_ENDPOINT.into()),
            model: env::var("EDGEONE_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into()),
            temperature: env::var("NEOMUX_TEMPERATURE")
                .ok()
                .and_then(|value| value.parse().ok())
                .or(Some(0.2)),
            max_tokens: env::var("NEOMUX_MAX_TOKENS")
                .ok()
                .and_then(|value| value.parse().ok()),
            files: Vec::new(),
            commands: VecDeque::new(),
            conversation: Vec::new(),
        }
    }

    fn push_command(&mut self, label: String, mut result: CommandResult) {
        result.label = label;
        self.commands.push_back(result);
        while self.commands.len() > 8 {
            self.commands.pop_front();
        }
    }

    fn chat_body(&self, prompt: &str) -> String {
        let mut messages = vec![("system".to_string(), PROMPT.to_string())];
        let context = self.context_prompt();
        if !context.is_empty() {
            messages.push(("user".into(), context));
        }
        messages.extend(self.conversation.iter().cloned());
        messages.push(("user".into(), prompt.to_string()));

        let mut body = format!(
            "{{\"model\":\"{}\",\"stream\":true,\"messages\":[",
            json(&self.model)
        );
        for (index, (role, content)) in messages.iter().enumerate() {
            if index > 0 {
                body.push(',');
            }
            body.push_str(&format!(
                "{{\"role\":\"{}\",\"content\":\"{}\"}}",
                json(role),
                json(content)
            ));
        }
        body.push(']');
        if let Some(temperature) = self.temperature {
            body.push_str(&format!(",\"temperature\":{temperature}"));
        }
        if let Some(max_tokens) = self.max_tokens {
            body.push_str(&format!(",\"max_tokens\":{max_tokens}"));
        }
        body.push('}');
        body
    }

    fn context_prompt(&self) -> String {
        let mut text = String::new();
        if !self.files.is_empty() {
            text.push_str("Loaded files:\n");
            for file in &self.files {
                text.push_str(&format!("--- {}\n{}\n\n", file.path, file.text));
            }
        }
        if !self.commands.is_empty() {
            text.push_str("Recent commands:\n");
            for command in self.commands.iter().rev().take(5) {
                text.push_str(&format!(
                    "$ {}\nexit {} in {}ms\n{}\n\n",
                    command.label, command.code, command.ms, command.text
                ));
            }
        }
        text
    }
}

struct LoadedFile {
    path: String,
    text: String,
}

struct CommandResult {
    label: String,
    text: String,
    code: i32,
    ms: u128,
}

fn doctor() -> Result<(), String> {
    let mut failed = false;
    for command in ["tmux", "nvim", "rg", "curl"] {
        if command_exists(command) {
            println!("ok   {command}");
        } else {
            failed = true;
            println!("miss {command}  fix: install {command} and make sure it is in PATH");
        }
    }
    if env::var("MAKERS_MODELS_KEY").is_ok() {
        println!("ok   MAKERS_MODELS_KEY");
    } else {
        failed = true;
        println!("miss MAKERS_MODELS_KEY  fix: export MAKERS_MODELS_KEY=\"...\"");
    }
    if env::var("COLORTERM").is_ok()
        || env::var("TERM")
            .is_ok_and(|term| term.contains("256color") || term.contains("truecolor"))
    {
        println!("ok   terminal colors");
    } else {
        println!("warn terminal colors  fix: use a 256-color or truecolor terminal");
    }
    let endpoint = env::var("EDGEONE_BASE_URL").unwrap_or_else(|_| DEFAULT_ENDPOINT.into());
    if endpoint_reachable(&endpoint) {
        println!("ok   EdgeOne endpoint");
    } else {
        println!("warn EdgeOne endpoint  fix: check network or EDGEONE_BASE_URL");
    }
    if failed {
        io::stdout().flush().map_err(|error| error.to_string())?;
        return Err("doctor found missing requirements".into());
    }
    Ok(())
}

fn models() -> Result<(), String> {
    println!(
        "{:<30} {:<10} {:<12} pricing",
        "model", "vendor", "streaming"
    );
    println!(
        "{:<30} {:<10} {:<12} unknown/unverified",
        "@makers/deepseek-v4-flash", "DeepSeek", "yes"
    );
    println!(
        "{:<30} {:<10} {:<12} unknown/unverified",
        "@makers/hunyuan-turbos-latest", "Hunyuan", "yes"
    );
    Ok(())
}

fn config() -> Result<(), String> {
    println!(
        "endpoint={}",
        env::var("EDGEONE_BASE_URL").unwrap_or_else(|_| DEFAULT_ENDPOINT.into())
    );
    println!(
        "model={}",
        env::var("EDGEONE_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into())
    );
    println!(
        "key={}",
        if env::var("MAKERS_MODELS_KEY").is_ok() {
            "set"
        } else {
            "missing"
        }
    );
    Ok(())
}

fn version(json_output: bool) -> Result<(), String> {
    let target = format!("{}-{}", env::consts::ARCH, env::consts::OS);
    if json_output {
        println!(
            "{{\"version\":\"{}\",\"target\":\"{}\",\"defaultModel\":\"{}\",\"edgeoneBaseUrl\":\"{}\"}}",
            VERSION, target, DEFAULT_MODEL, DEFAULT_ENDPOINT
        );
    } else {
        println!("neomux {VERSION}");
    }
    Ok(())
}

fn split_command(line: &str) -> (&str, &str) {
    line.split_once(' ')
        .map_or((line, ""), |(command, arg)| (command, arg.trim()))
}

fn run_shell(command: &str, cwd: &Path) -> Result<CommandResult, String> {
    run_capture("sh", &["-lc", command], cwd)
}

fn run_capture(program: &str, args: &[&str], cwd: &Path) -> Result<CommandResult, String> {
    let started = Instant::now();
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|error| error.to_string())?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(CommandResult {
        label: program.into(),
        text,
        code: output.status.code().unwrap_or(1),
        ms: started.elapsed().as_millis(),
    })
}

fn risk(command: &str) -> Option<&'static str> {
    let lower = command.to_lowercase();
    if lower.contains("rm -rf") || lower.contains("rm -fr") {
        Some("recursive force remove")
    } else if lower.contains("sudo ") {
        Some("privileged command")
    } else if lower.contains("git reset") || lower.contains("git clean") {
        Some("destructive git command")
    } else {
        None
    }
}

fn sse_content(line: &str) -> Option<String> {
    let data = line.strip_prefix("data: ")?.trim();
    if data == "[DONE]" {
        return None;
    }
    let key = "\"content\":\"";
    let start = data.find(key)? + key.len();
    let mut escaped = false;
    let mut raw = String::new();
    for ch in data[start..].chars() {
        if escaped {
            raw.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            break;
        } else {
            raw.push(ch);
        }
    }
    Some(raw)
}

fn json(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            other => vec![other],
        })
        .collect()
}

fn require_command(command: &str) -> Result<(), String> {
    if command_exists(command) {
        Ok(())
    } else {
        Err(format!(
            "{command} is required. Fix: install {command} and make sure it is in PATH"
        ))
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .args(["-lc", &format!("command -v {command}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn endpoint_reachable(endpoint: &str) -> bool {
    Command::new("curl")
        .args(["-sS", "--max-time", "5", "-o", "/dev/null", endpoint])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn tmux<const N: usize>(args: [&str; N]) -> Result<(), String> {
    let status = Command::new("tmux")
        .args(args)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux failed with status {status}"))
    }
}

fn tmux_output<const N: usize>(args: [&str; N]) -> Result<String, String> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn path_str(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| "path is not valid utf-8".into())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}
