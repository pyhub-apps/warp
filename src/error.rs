use thiserror::Error;

#[derive(Debug, Error)]
pub enum WarpError {
    #[error("🔑 API 키가 설정되지 않았습니다")]
    NoApiKey,

    #[error("🌐 네트워크 오류가 발생했습니다")]
    Network(#[from] reqwest::Error),

    #[error("⚠️ API 오류 ({code}): {message}")]
    ApiError {
        code: String,
        message: String,
        hint: Option<String>,
    },

    #[error("⚙️ 설정 오류: {0}")]
    Config(String),

    #[error("💾 파일 시스템 오류가 발생했습니다")]
    Io(#[from] std::io::Error),

    #[error("📄 데이터 변환 오류가 발생했습니다")]
    Serialization(#[from] serde_json::Error),

    #[error("📊 CSV 처리 오류가 발생했습니다")]
    Csv(#[from] csv::Error),

    #[error("🔍 응답 파싱 오류: {0}")]
    Parse(String),

    #[error("❌ 잘못된 입력: {0}")]
    InvalidInput(String),

    #[error("🔎 찾을 수 없음: {0}")]
    #[allow(dead_code)]
    NotFound(String),

    #[error("⏱️ 시간 초과: {0}초 후 작업이 중단되었습니다")]
    #[allow(dead_code)]
    Timeout(u64),

    #[error("⏳ 요청 한도 초과")]
    RateLimit,

    #[error("💾 캐시 오류: {0}")]
    Cache(String),

    #[error("🚨 서버 오류: {0}")]
    ServerError(String),

    #[error("🔐 인증 실패: {0}")]
    #[allow(dead_code)]
    AuthenticationFailed(String),

    #[error("⚠️ {0}")]
    Other(String),
}

impl WarpError {
    /// Create an API error with an optional hint
    #[allow(dead_code)]
    pub fn api_error(code: impl Into<String>, message: impl Into<String>, hint: Option<String>) -> Self {
        Self::ApiError {
            code: code.into(),
            message: message.into(),
            hint,
        }
    }

    /// Get user-friendly hint for the error
    #[allow(dead_code)]
    pub fn hint(&self) -> Option<String> {
        match self {
            Self::NoApiKey => Some(
                "💡 해결 방법:\n\
                 1. https://open.law.go.kr 에서 API 키를 발급받으세요\n\
                 2. 다음 명령어로 API 키를 설정하세요:\n\
                    warp config set law.nlic.key YOUR_API_KEY".to_string()
            ),
            Self::ApiError { hint, .. } => hint.clone(),
            Self::Network(e) => {
                let mut hint = String::from("💡 해결 방법:\n");
                if e.is_timeout() {
                    hint.push_str("• 네트워크 연결이 느립니다. 잠시 후 다시 시도해주세요\n");
                } else if e.is_connect() {
                    hint.push_str("• 인터넷 연결을 확인해주세요\n");
                    hint.push_str("• 방화벽이나 프록시 설정을 확인해주세요\n");
                } else {
                    hint.push_str("• 인터넷 연결 상태를 확인해주세요\n");
                    hint.push_str("• 문제가 지속되면 잠시 후 다시 시도해주세요\n");
                }
                Some(hint)
            },
            Self::RateLimit => Some(
                "💡 해결 방법:\n\
                 • API 요청 한도를 초과했습니다\n\
                 • 잠시 후(약 1분) 다시 시도해주세요\n\
                 • 빈번한 요청은 피해주세요".to_string()
            ),
            Self::Cache(_) => Some(
                "💡 해결 방법:\n\
                 • 캐시 디렉토리의 권한을 확인해주세요\n\
                 • warp cache clear 명령으로 캐시를 초기화해보세요\n\
                 • 디스크 공간이 충분한지 확인하세요".to_string()
            ),
            Self::AuthenticationFailed(_) => Some(
                "💡 해결 방법:\n\
                 • API 키가 올바른지 확인해주세요\n\
                 • warp config get law.nlic.key 명령으로 현재 설정을 확인하세요\n\
                 • 키가 만료되었다면 새로 발급받으세요".to_string()
            ),
            Self::Parse(msg) if msg.contains("XML") || msg.contains("JSON") => Some(
                "💡 해결 방법:\n\
                 • API 응답 형식이 예상과 다릅니다\n\
                 • --verbose 옵션으로 자세한 정보를 확인하세요\n\
                 • 문제가 지속되면 GitHub에 이슈를 등록해주세요".to_string()
            ),
            Self::InvalidInput(_) => Some(
                "💡 해결 방법:\n\
                 • 입력한 값을 다시 확인해주세요\n\
                 • warp --help 명령으로 사용법을 확인하세요".to_string()
            ),
            Self::NotFound(item) => Some(
                format!("💡 해결 방법:\n\
                 • '{}' 항목을 찾을 수 없습니다\n\
                 • 검색어나 ID를 다시 확인해주세요\n\
                 • 다른 검색어로 시도해보세요", item)
            ),
            Self::ServerError(_) => Some(
                "💡 해결 방법:\n\
                 • 서버에 일시적인 문제가 있습니다\n\
                 • 잠시 후 다시 시도해주세요\n\
                 • 문제가 지속되면 https://www.law.go.kr 서비스 상태를 확인하세요".to_string()
            ),
            Self::Io(e) => {
                let mut hint = String::from("💡 해결 방법:\n");
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    hint.push_str("• 파일 또는 디렉토리에 대한 권한이 없습니다\n");
                    hint.push_str("• sudo 명령어를 사용하거나 권한을 확인하세요\n");
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    hint.push_str("• 파일 또는 디렉토리를 찾을 수 없습니다\n");
                    hint.push_str("• 경로를 다시 확인해주세요\n");
                } else {
                    hint.push_str("• 파일 시스템 오류가 발생했습니다\n");
                    hint.push_str("• 디스크 공간과 권한을 확인하세요\n");
                }
                Some(hint)
            },
            _ => None,
        }
    }

    /// Check if the error is retryable
    #[allow(dead_code)]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout(_) | Self::ServerError(_) | Self::RateLimit
        )
    }
}

pub type Result<T> = std::result::Result<T, WarpError>;