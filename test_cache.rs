use chrono::Duration;
use std::path::PathBuf;
use warp::api::ApiType;
use warp::cache::{CacheConfig, CacheStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 캐시 동작 테스트 ===\n");

    // 캐시 설정
    let config = CacheConfig {
        max_size: 10 * 1024 * 1024, // 10MB
        default_ttl: Duration::seconds(3600),
        db_path: PathBuf::from("/Users/allieus/Library/Caches/pyhub-warp/cache.db"),
    };

    // 캐시 저장소 생성
    let cache = CacheStore::new(config).await?;
    println!("✅ 캐시 저장소 초기화 완료\n");

    // 테스트 데이터 저장
    let test_key = "test_민법_검색";
    let test_data = "{'results': ['민법 제1조', '민법 제2조']}"
        .as_bytes()
        .to_vec();

    println!("📝 데이터 저장 중...");
    cache
        .put(test_key, test_data.clone(), ApiType::Nlic, None)
        .await?;
    println!("✅ 캐시에 데이터 저장 완료: key={}\n", test_key);

    // 데이터 조회
    println!("🔍 캐시에서 데이터 조회 중...");
    if let Some(cached_data) = cache.get(test_key).await? {
        println!("✅ 캐시 HIT! 데이터 크기: {} bytes", cached_data.len());
        println!("   내용: {}", String::from_utf8_lossy(&cached_data));
    } else {
        println!("❌ 캐시 MISS");
    }

    // 통계 확인
    println!("\n📊 캐시 통계:");
    let stats = cache.stats().await?;
    println!("   총 항목수: {}", stats.total_entries);
    println!("   총 크기: {} bytes", stats.total_size);
    println!("   만료 항목: {}", stats.expired_entries);

    // API별 캐시 삭제 테스트
    println!("\n🗑️  NLIC API 캐시 삭제 중...");
    cache.clear_api(ApiType::Nlic).await?;
    println!("✅ NLIC 캐시 삭제 완료");

    // 삭제 후 조회
    println!("\n🔍 삭제 후 재조회...");
    if let Some(_) = cache.get(test_key).await? {
        println!("❌ 데이터가 여전히 존재함");
    } else {
        println!("✅ 데이터가 성공적으로 삭제됨");
    }

    Ok(())
}
