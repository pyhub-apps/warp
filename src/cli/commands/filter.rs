use crate::cli::args::{FilterArgs, FilterCommand};
use crate::config::{Config, FilterPreset};
use crate::error::{Result, WarpError};
use chrono::Utc;
use std::io::{self, Write};

/// Execute filter preset management commands
pub async fn execute(args: FilterArgs) -> Result<()> {
    match args.command {
        FilterCommand::Save {
            name,
            query,
            law_type,
            department,
            status,
            region,
            from,
            to,
            recent_days,
            regex,
            title_only,
            min_score,
        } => {
            execute_save_command(
                name,
                query,
                law_type,
                department,
                status,
                region,
                from,
                to,
                recent_days,
                regex,
                title_only,
                min_score,
            )
            .await
        }
        FilterCommand::List => execute_list_command().await,
        FilterCommand::Show { name } => execute_show_command(name).await,
        FilterCommand::Delete { name, force } => execute_delete_command(name, force).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_save_command(
    name: String,
    query: Option<String>,
    law_type: Option<String>,
    department: Option<String>,
    status: Option<String>,
    region: Option<String>,
    from: Option<String>,
    to: Option<String>,
    recent_days: Option<u32>,
    regex: bool,
    title_only: bool,
    min_score: Option<f32>,
) -> Result<()> {
    let mut config = Config::load()?;

    // Check if preset already exists
    if config.filter_presets.contains_key(&name) {
        print!(
            "🔄 프리셋 '{}'이 이미 존재합니다. 덮어쓰시겠습니까? (y/N): ",
            name
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ 취소되었습니다.");
            return Ok(());
        }
    }

    // Create new filter preset
    let preset = FilterPreset {
        name: name.clone(),
        query,
        law_type,
        department,
        status,
        region,
        from,
        to,
        recent_days,
        regex,
        title_only,
        min_score,
        created_at: Utc::now(),
    };

    // Save preset
    config.filter_presets.insert(name.clone(), preset);
    config.save()?;

    println!("✅ 필터 프리셋 '{}'이 저장되었습니다.", name);
    Ok(())
}

async fn execute_list_command() -> Result<()> {
    let config = Config::load()?;

    if config.filter_presets.is_empty() {
        println!("📭 저장된 필터 프리셋이 없습니다.");
        return Ok(());
    }

    println!("🔍 저장된 필터 프리셋:");
    println!("{}", "─".repeat(50));

    for (name, preset) in &config.filter_presets {
        println!("📌 {}", name);

        if let Some(ref query) = preset.query {
            println!("   검색어: {}", query);
        }

        let mut filters = Vec::new();
        if let Some(ref law_type) = preset.law_type {
            filters.push(format!("법령종류: {}", law_type));
        }
        if let Some(ref department) = preset.department {
            filters.push(format!("부처: {}", department));
        }
        if let Some(ref status) = preset.status {
            filters.push(format!("상태: {}", status));
        }
        if preset.regex {
            filters.push("정규표현식".to_string());
        }
        if preset.title_only {
            filters.push("제목만".to_string());
        }

        if !filters.is_empty() {
            println!("   필터: {}", filters.join(", "));
        }

        println!(
            "   생성일: {}",
            preset.created_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!();
    }

    Ok(())
}

async fn execute_show_command(name: String) -> Result<()> {
    let config = Config::load()?;

    let preset = config
        .filter_presets
        .get(&name)
        .ok_or_else(|| WarpError::InvalidInput(format!("프리셋 '{}'을 찾을 수 없습니다", name)))?;

    println!("🔍 필터 프리셋: {}", preset.name);
    println!("{}", "─".repeat(50));

    if let Some(ref query) = preset.query {
        println!("검색어: {}", query);
    }
    if let Some(ref law_type) = preset.law_type {
        println!("법령 종류: {}", law_type);
    }
    if let Some(ref department) = preset.department {
        println!("부처: {}", department);
    }
    if let Some(ref status) = preset.status {
        println!("상태: {}", status);
    }
    if let Some(ref region) = preset.region {
        println!("지역: {}", region);
    }
    if let Some(ref from) = preset.from {
        println!("시작 날짜: {}", from);
    }
    if let Some(ref to) = preset.to {
        println!("종료 날짜: {}", to);
    }
    if let Some(recent_days) = preset.recent_days {
        println!("최근 {} 일", recent_days);
    }
    if preset.regex {
        println!("정규표현식: 활성화");
    }
    if preset.title_only {
        println!("제목만 검색: 활성화");
    }
    if let Some(min_score) = preset.min_score {
        println!("최소 점수: {}", min_score);
    }

    println!("생성일: {}", preset.created_at.format("%Y-%m-%d %H:%M:%S"));

    // Show usage example
    println!("\n💡 사용 예시:");
    println!("   warp search --filter {}", name);

    Ok(())
}

async fn execute_delete_command(name: String, force: bool) -> Result<()> {
    let mut config = Config::load()?;

    if !config.filter_presets.contains_key(&name) {
        return Err(WarpError::InvalidInput(format!(
            "프리셋 '{}'을 찾을 수 없습니다",
            name
        )));
    }

    if !force {
        print!("🗑️  필터 프리셋 '{}'을 삭제하시겠습니까? (y/N): ", name);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ 취소되었습니다.");
            return Ok(());
        }
    }

    config.filter_presets.remove(&name);
    config.save()?;

    println!("✅ 필터 프리셋 '{}'이 삭제되었습니다.", name);
    Ok(())
}
