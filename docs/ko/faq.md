# 자주 묻는 질문 (FAQ)

## 🔑 API 키 관련

### Q: API 키는 어디서 발급받나요?
**A:** [https://open.law.go.kr](https://open.law.go.kr) 에서 무료로 발급받을 수 있습니다.
1. 회원가입 또는 로그인
2. 마이페이지 → API 신청
3. 사용 목적 입력 후 신청
4. 즉시 발급 (무료, 일일 1,000회 제한)

### Q: API 키를 여러 개 사용할 수 있나요?
**A:** 네, 서비스별로 다른 API 키를 설정할 수 있습니다.
```bash
# 기본 API 키 (모든 서비스에 적용)
warp config set law.key DEFAULT_KEY

# 특정 서비스용 API 키
warp config set law.nlic.key NLIC_KEY
warp config set law.elis.key ELIS_KEY
```

### Q: API 키가 노출되면 어떻게 하나요?
**A:** 즉시 새 키를 발급받고 기존 키를 폐기하세요.
1. [open.law.go.kr](https://open.law.go.kr) 에서 새 키 발급
2. 기존 키 폐기
3. Warp에 새 키 설정: `warp config set law.key NEW_KEY`

## 🔍 검색 관련

### Q: 검색 결과가 너무 많아요. 어떻게 제한하나요?
**A:** `--size` 옵션을 사용하세요.
```bash
# 결과를 10개로 제한
warp law "민법" --size 10

# 특정 페이지 조회
warp law "민법" --page 2 --size 20
```

### Q: 특정 기간의 법령만 검색하고 싶어요.
**A:** 날짜 옵션을 사용하세요.
```bash
# 2024년 이후 제정/개정된 법령
warp law "개인정보" --date-from 20240101

# 특정 기간
warp law "민법" --date-from 20240101 --date-to 20241231
```

### Q: 여러 API를 동시에 검색할 수 있나요?
**A:** `search` 명령어를 사용하세요.
```bash
# 모든 소스에서 검색
warp search "개인정보보호"

# 특정 소스만 선택
warp search "민법" --source nlic,elis,prec
```

### Q: 검색 결과가 없다고 나와요.
**A:** 다음을 확인해보세요:
1. 검색어를 더 간단하게 (예: "개인정보보호법" → "개인정보")
2. 다른 API 서비스 시도
3. 통합 검색 사용: `warp search "검색어"`

## 📄 출력 형식 관련

### Q: Excel에서 CSV 파일이 깨져요.
**A:** Warp는 BOM이 포함된 UTF-8 CSV를 생성합니다.
- **방법 1**: Excel에서 "데이터 → 텍스트/CSV에서 가져오기" 사용
- **방법 2**: 메모장으로 열어서 다시 저장 후 Excel에서 열기

### Q: JSON 결과를 어떻게 처리하나요?
**A:** `jq` 같은 도구를 사용하세요.
```bash
# 제목만 추출
warp law "민법" --format json | jq '.items[] | .title'

# 특정 필드 필터링
warp law "민법" --format json | jq '.items[] | {title: .title, date: .date}'

# Python으로 처리
warp law "민법" --format json | python -c "import json,sys; data=json.load(sys.stdin); print(data['total'])"
```

### Q: Markdown 출력을 어디에 사용하나요?
**A:** 문서 작성이나 GitHub에 유용합니다.
```bash
# README에 추가
warp law "저작권법" --format markdown >> README.md

# GitHub 이슈에 붙여넣기용
warp law "개인정보" --format markdown --size 5
```

## ⚙️ 설정 관련

### Q: 설정 파일은 어디에 저장되나요?
**A:** 운영체제별로 다릅니다.
- **Linux/macOS**: `~/.config/warp/config.yaml`
- **Windows**: `%APPDATA%\warp\config.yaml`

확인 방법:
```bash
warp config path
```

### Q: 설정을 초기화하려면?
**A:** 
```bash
# 설정 파일 삭제
rm ~/.config/warp/config.yaml

# 재초기화
warp config init
```

### Q: 설정을 백업하려면?
**A:**
```bash
# 백업
cp ~/.config/warp/config.yaml ~/warp-config-backup.yaml

# 복원
cp ~/warp-config-backup.yaml ~/.config/warp/config.yaml
```

## 🚀 성능 관련

### Q: 검색이 느려요.
**A:** 다음을 시도해보세요:
1. 결과 수 줄이기: `--size 10`
2. 특정 소스만 검색: `--source nlic`
3. 네트워크 연결 확인

### Q: 메모리를 많이 사용해요.
**A:** 대량의 결과를 요청하면 메모리 사용량이 증가합니다.
- 페이지네이션 사용: `--page 1 --size 20`
- JSON 대신 CSV 형식 사용 (메모리 효율적)

## 🛠️ 개발 관련

### Q: 스크립트에서 Warp를 사용하려면?
**A:** JSON 출력을 파이프로 연결하세요.
```bash
#!/bin/bash
# 법령 ID 추출
LAW_ID=$(warp law "민법" --format json | jq -r '.items[0].id')

# Python 스크립트
import subprocess
import json

result = subprocess.run(['warp', 'law', '민법', '--format', 'json'], 
                       capture_output=True, text=True)
data = json.loads(result.stdout)
```

### Q: CI/CD에서 사용할 수 있나요?
**A:** 네, 환경 변수로 API 키를 설정하세요.
```yaml
# GitHub Actions 예제
- name: Search laws
  env:
    LAW_API_KEY: ${{ secrets.LAW_API_KEY }}
  run: |
    warp config set law.key $LAW_API_KEY
    warp law "개인정보" --format json > results.json
```

## 🐛 문제 해결

### Q: "Command not found" 오류가 나요.
**A:** PATH에 warp가 없습니다.
```bash
# 설치 확인
which warp

# PATH에 추가
export PATH="/usr/local/bin:$PATH"
```

### Q: 설치했는데 실행이 안 돼요.
**A:** 실행 권한을 확인하세요.
```bash
chmod +x /usr/local/bin/warp
```

### Q: 특정 명령어가 작동하지 않아요.
**A:** 버전을 확인하고 업데이트하세요.
```bash
# 버전 확인
warp --version

# 업데이트 (Cargo)
cargo install warp --force
```

## 📚 기타

### Q: 오프라인에서 사용할 수 있나요?
**A:** 아니요, API 호출을 위해 인터넷 연결이 필요합니다.

### Q: 캐싱 기능이 있나요?
**A:** 현재는 없지만 개발 예정입니다.

### Q: 다른 언어 버전도 있나요?
**A:** 네, 있습니다:
- [Python 버전](https://github.com/pyhub-kr/pyhub-warp-cli) (원본)
- [Go 버전](https://github.com/pyhub-apps/warp-golang)
- Rust 버전 (현재 문서)

### Q: 기여하고 싶어요.
**A:** 환영합니다!
1. [GitHub 저장소](https://github.com/pyhub-apps/warp) Fork
2. 기능 브랜치 생성
3. 변경사항 커밋
4. Pull Request 제출

### Q: 라이선스는 무엇인가요?
**A:** MIT 라이선스입니다. 자유롭게 사용, 수정, 배포할 수 있습니다.

## 📮 추가 질문

여기에 없는 질문은:
- **GitHub Issues**: [https://github.com/pyhub-apps/warp/issues](https://github.com/pyhub-apps/warp/issues)
- **이메일**: pyhub.kr@gmail.com

## 📖 관련 문서

- [설치 가이드](installation.md)
- [문제 해결](troubleshooting.md)
- [사용 예제](examples.md)