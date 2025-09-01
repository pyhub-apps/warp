# 문제 해결 가이드

Warp CLI 사용 중 발생할 수 있는 일반적인 문제와 해결 방법을 안내합니다.

## 🔑 API 키 관련 문제

### "API 키가 설정되지 않았습니다" 오류

**증상**
```
Error: API key not configured for law
```

**해결 방법**
```bash
# 1. API 키 발급 확인
# https://open.law.go.kr 에서 API 키 발급

# 2. API 키 설정
warp config set law.key YOUR_API_KEY

# 3. 설정 확인
warp config get law.key

# 4. 설정 파일 직접 확인
cat ~/.config/warp/config.yaml
```

### API 인증 실패

**증상**
```
Error: Authentication failed (401 Unauthorized)
```

**원인 및 해결**
1. **잘못된 API 키**
   ```bash
   # API 키 재설정
   warp config set law.key CORRECT_API_KEY
   ```

2. **API 키 만료**
   - [open.law.go.kr](https://open.law.go.kr) 에서 새 키 발급
   - 일일 호출 한도 초과 여부 확인

3. **서비스별 다른 API 키 필요**
   ```bash
   # ELIS는 별도 키 필요
   warp config set law.elis.key ELIS_API_KEY
   ```

### 설정 파일 권한 오류

**증상**
```
Error: Permission denied accessing config file
```

**해결 방법**
```bash
# 설정 파일 권한 확인
ls -la ~/.config/warp/config.yaml

# 권한 수정 (보안을 위해 0600 권장)
chmod 600 ~/.config/warp/config.yaml
```

## 🌐 네트워크 관련 문제

### 연결 시간 초과

**증상**
```
Error: Request timeout after 30 seconds
```

**해결 방법**
1. **인터넷 연결 확인**
   ```bash
   # API 서버 접속 테스트
   curl -I https://open.law.go.kr
   ```

2. **프록시 설정** (기업 네트워크의 경우)
   ```bash
   # 환경 변수 설정
   export HTTP_PROXY=http://proxy.company.com:8080
   export HTTPS_PROXY=http://proxy.company.com:8080
   ```

3. **방화벽 확인**
   - 443 포트 (HTTPS) 접근 허용 필요
   - API 도메인 화이트리스트 추가

### SSL 인증서 오류

**증상**
```
Error: SSL certificate verification failed
```

**해결 방법**
```bash
# 시스템 인증서 업데이트 (Linux)
sudo update-ca-certificates

# macOS
brew install ca-certificates
```

## 🔍 검색 결과 문제

### 검색 결과가 없음

**증상**
```
No results found for query: "검색어"
```

**확인 사항**
1. **검색어 확인**
   ```bash
   # 더 일반적인 검색어 사용
   warp law "민법" # 구체적
   warp law "민" # 더 포괄적
   ```

2. **API 서비스 상태**
   - 일부 API (ADMRUL, EXPC)는 특정 검색어에 결과 없을 수 있음
   - 다른 API 서비스 시도

3. **검색 옵션 조정**
   ```bash
   # 페이지 크기 늘리기
   warp law "검색어" --size 50
   
   # 통합 검색 사용
   warp search "검색어" --source all
   ```

### 페이지네이션 문제

**증상**
```
2페이지 이상 조회 시 결과 없음
```

**알려진 이슈**
- 일부 API의 페이지네이션 구현 문제
- 임시 해결책: `--size` 옵션으로 첫 페이지 결과 늘리기

```bash
# 더 많은 결과를 한 번에 가져오기
warp law "민법" --size 100
```

### 파싱 오류

**증상**
```
Error: Failed to parse API response
```

**원인**
- API가 단일 결과를 객체로, 복수 결과를 배열로 반환하는 불일치

**해결 방법**
```bash
# 디버그 모드로 실행하여 원시 응답 확인
RUST_LOG=debug warp law "검색어" --verbose

# 다른 출력 형식 시도
warp law "검색어" --format json
```

## 📄 출력 형식 문제

### CSV 파일 인코딩 문제

**증상**
- Excel에서 CSV 파일 열 때 한글 깨짐

**해결 방법**
```bash
# Warp는 BOM이 포함된 UTF-8 CSV 생성
warp law "민법" --format csv > result.csv

# Excel에서 올바르게 열기:
# 1. Excel 실행
# 2. 데이터 → 텍스트/CSV에서 가져오기
# 3. 파일 선택 후 UTF-8 인코딩 선택
```

### JSON 출력 파싱

**증상**
- JSON 출력을 다른 도구에서 처리할 때 오류

**해결 방법**
```bash
# jq를 사용한 JSON 처리
warp law "민법" --format json | jq '.items[] | .title'

# Python으로 처리
warp law "민법" --format json | python -m json.tool
```

### HTML 렌더링 문제

**증상**
- 브라우저에서 HTML 출력이 제대로 표시되지 않음

**해결 방법**
```bash
# 완전한 HTML 문서 생성
warp law "민법" --format html > result.html

# 간단한 HTML 조각만 필요한 경우
warp law "민법" --format html-simple
```

## 🚀 성능 문제

### 검색 속도가 느림

**원인 및 해결**
1. **네트워크 지연**
   - 인터넷 연결 속도 확인
   - API 서버 응답 시간 확인

2. **너무 많은 결과 요청**
   ```bash
   # 결과 수 제한
   warp law "민법" --size 10
   ```

3. **통합 검색 시 병렬 처리**
   ```bash
   # 특정 소스만 검색하여 속도 향상
   warp search "검색어" --source nlic,elis
   ```

## 🛠️ 일반적인 오류 메시지

### "Command not found"

**해결 방법**
```bash
# PATH 확인
echo $PATH

# warp 위치 확인
which warp

# PATH에 추가 (bash)
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### "Permission denied"

**해결 방법**
```bash
# 실행 권한 부여
chmod +x /usr/local/bin/warp

# 소유권 확인
ls -la /usr/local/bin/warp
```

### "Config file not found"

**해결 방법**
```bash
# 설정 초기화
warp config init

# 설정 파일 위치 확인
warp config path
```

## 📊 디버깅 방법

### 상세 로그 활성화

```bash
# 디버그 로그 활성화
RUST_LOG=debug warp law "민법" --verbose

# 특정 모듈만 디버그
RUST_LOG=warp::api=debug warp law "민법"
```

### 버전 및 환경 정보 수집

```bash
# 버그 리포트 시 필요한 정보
warp --version
rustc --version
uname -a  # 운영체제 정보
```

## 🆘 추가 도움말

### 해결되지 않는 문제

1. **GitHub Issues 확인**
   - [기존 이슈](https://github.com/pyhub-apps/warp/issues) 검색
   - 유사한 문제가 없으면 새 이슈 생성

2. **이슈 작성 시 포함할 정보**
   - 실행한 명령어
   - 오류 메시지 전문
   - 운영체제 및 버전
   - Warp 버전 (`warp --version`)
   - 재현 단계

3. **커뮤니티 지원**
   - Email: pyhub.kr@gmail.com
   - GitHub Discussions (준비 중)

## 📚 관련 문서

- [설치 가이드](installation.md)
- [FAQ](faq.md)
- [사용 예제](examples.md)