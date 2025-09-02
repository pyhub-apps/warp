# Warp CLI 문서

한국 법령 정보 검색 CLI 도구인 Warp의 공식 문서입니다.

## 📚 문서 목록

### 시작하기

- [📦 설치 가이드](installation.md) - 시스템 요구사항 및 설치 방법
- [📖 사용자 가이드](user-guide.md) - 기본 사용법 및 명령어 설명

### 참고 자료

- [🔧 문제 해결](troubleshooting.md) - 일반적인 문제와 해결 방법
- [❓ 자주 묻는 질문](faq.md) - FAQ
- [💡 사용 예제](examples.md) - 실제 사용 시나리오별 예제

## 🚀 빠른 시작

### 1. 설치

```bash
cargo install warp
```

### 2. API 키 설정

```bash
warp config init
warp config set law.key YOUR_API_KEY
```

### 3. 첫 검색

```bash
warp law "민법"
```

## 🔗 유용한 링크

- [GitHub 저장소](https://github.com/pyhub-apps/warp)
- [이슈 트래커](https://github.com/pyhub-apps/warp/issues)
- [API 키 발급](https://open.law.go.kr)

## 📮 도움이 필요하신가요?

- **버그 리포트**: [GitHub Issues](https://github.com/pyhub-apps/warp/issues)
- **이메일**: pyhub.kr@gmail.com

## 🤝 기여하기

문서 개선에 기여하고 싶으시다면:

1. 저장소를 Fork 하세요
2. 문서를 수정하세요
3. Pull Request를 제출하세요

모든 기여를 환영합니다!
