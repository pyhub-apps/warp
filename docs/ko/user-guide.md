# 사용자 가이드

Warp CLI의 전체 기능과 사용법을 상세히 안내합니다.

## 📖 목차

1. [시작하기](#시작하기)
2. [명령어 구조](#명령어-구조)
3. [법령 검색 (law)](#법령-검색-law)
4. [판례 검색 (precedent)](#판례-검색-precedent)
5. [자치법규 검색 (ordinance)](#자치법규-검색-ordinance)
6. [행정규칙 검색 (admrule)](#행정규칙-검색-admrule)
7. [법령해석례 검색 (interpretation)](#법령해석례-검색-interpretation)
8. [통합 검색 (search)](#통합-검색-search)
9. [설정 관리 (config)](#설정-관리-config)
10. [출력 형식](#출력-형식)
11. [고급 기능](#고급-기능)

## 시작하기

### 첫 실행 전 준비

1. **API 키 발급**: [https://open.law.go.kr](https://open.law.go.kr)
2. **설정 초기화**:
   ```bash
   warp config init
   warp config set law.key YOUR_API_KEY
   ```
3. **동작 확인**:
   ```bash
   warp law "민법" --size 1
   ```

## 명령어 구조

### 기본 구조

```
warp [명령어] [검색어] [옵션]
```

### 전역 옵션

- `--help, -h`: 도움말 표시
- `--version, -V`: 버전 정보
- `--verbose`: 상세 출력 모드

### 명령어 목록

- `law`: 국가법령정보센터 검색
- `ordinance`: 자치법규 검색
- `precedent`: 판례 검색
- `admrule`: 행정규칙 검색
- `interpretation`: 법령해석례 검색
- `search`: 통합 검색
- `config`: 설정 관리

## 법령 검색 (law)

국가법령정보센터(NLIC)의 법령 정보를 검색합니다.

### 기본 사용법

```bash
warp law [검색어] [옵션]
```

### 옵션

- `--page, -p`: 페이지 번호 (기본: 1)
- `--size, -s`: 페이지당 결과 수 (기본: 10)
- `--format, -f`: 출력 형식 (table|json|markdown|csv|html|html-simple)
- `--date-from`: 시작 날짜 (YYYYMMDD)
- `--date-to`: 종료 날짜 (YYYYMMDD)

### 예제

```bash
# 기본 검색
warp law "개인정보보호법"

# 결과 제한
warp law "민법" --size 5

# 특정 기간
warp law "개인정보" --date-from 20240101 --date-to 20241231

# JSON 출력
warp law "저작권법" --format json
```

### 하위 명령어

#### 상세 정보 조회

```bash
warp law detail [법령ID]
```

#### 개정 이력 조회

```bash
warp law history [법령ID]
```

## 판례 검색 (precedent)

법원 판례를 검색합니다.

### 기본 사용법

```bash
warp precedent [검색어] [옵션]
```

### 옵션

- `--court`: 법원 지정 (대법원, 고등법원, 지방법원 등)
- `--case-type`: 사건 유형
- `--date-from`: 시작 날짜
- `--date-to`: 종료 날짜
- `--page, --size, --format`: 공통 옵션

### 예제

```bash
# 대법원 판례
warp precedent "음주운전" --court "대법원"

# 특정 기간 판례
warp precedent "개인정보" --date-from 20230101

# 상세 조회
warp precedent detail CASE_ID
```

## 자치법규 검색 (ordinance)

지방자치단체의 조례 및 규칙을 검색합니다.

### 기본 사용법

```bash
warp ordinance [검색어] [옵션]
```

### 옵션

- `--region`: 지역 지정
- `--type`: 자치법규 유형 (조례, 규칙)
- 공통 옵션 지원

### 예제

```bash
# 서울시 조례
warp ordinance "서울 주차"

# 특정 구 조례
warp ordinance "강남구"

# 상세 조회
warp ordinance detail ORD_ID
```

## 행정규칙 검색 (admrule)

중앙행정기관의 행정규칙을 검색합니다.

### 기본 사용법

```bash
warp admrule [검색어] [옵션]
```

### 옵션

- `--department`: 부처 지정
- `--rule-type`: 규칙 유형
- 공통 옵션 지원

### 예제

```bash
# 행정안전부 규칙
warp admrule "개인정보" --department "행정안전부"

# 모든 부처
warp admrule "정보공개"
```

## 법령해석례 검색 (interpretation)

법령 해석 사례를 검색합니다.

### 기본 사용법

```bash
warp interpretation [검색어] [옵션]
```

### 예제

```bash
# 건축법 해석례
warp interpretation "건축법"

# 결과 제한
warp interpretation "세법" --size 5
```

## 통합 검색 (search)

모든 API를 동시에 검색합니다.

### 기본 사용법

```bash
warp search [검색어] [옵션]
```

### 옵션

- `--source`: 검색할 소스 지정 (nlic,elis,prec,admrul,expc)
- `--parallel`: 병렬 처리 (기본: true)
- 공통 옵션 지원

### 예제

```bash
# 모든 소스 검색
warp search "개인정보보호"

# 특정 소스만
warp search "민법" --source nlic,prec

# CSV로 저장
warp search "환경" --format csv > result.csv
```

## 설정 관리 (config)

### 하위 명령어

#### 초기화

```bash
warp config init
```

#### 설정 값 조회

```bash
warp config get [키]

# 예제
warp config get law.key
warp config get law.nlic.key
```

#### 설정 값 변경

```bash
warp config set [키] [값]

# 예제
warp config set law.key NEW_API_KEY
warp config set law.timeout 60
```

#### 설정 파일 경로

```bash
warp config path
```

### 설정 키 목록

- `law.key`: 기본 API 키
- `law.nlic.key`: NLIC 전용 API 키
- `law.elis.key`: ELIS 전용 API 키
- `law.prec.key`: PREC 전용 API 키
- `law.admrul.key`: ADMRUL 전용 API 키
- `law.expc.key`: EXPC 전용 API 키
- `law.timeout`: 요청 타임아웃 (초)
- `law.retry`: 재시도 횟수

## 출력 형식

### Table (기본)

```bash
warp law "민법"
```

터미널에 표 형식으로 출력

### JSON

```bash
warp law "민법" --format json
```

프로그래밍 연동에 적합

### Markdown

```bash
warp law "민법" --format markdown
```

문서 작성에 적합

### CSV

```bash
warp law "민법" --format csv
```

Excel 호환 (BOM 포함 UTF-8)

### HTML

```bash
warp law "민법" --format html
```

완전한 HTML 문서

### HTML Simple

```bash
warp law "민법" --format html-simple
```

HTML 조각 (임베딩용)

## 고급 기능

### 파이프라인 활용

```bash
# jq로 JSON 처리
warp law "민법" --format json | jq '.items[0]'

# grep으로 필터링
warp law "개인정보" | grep "보호"

# 파일로 저장
warp law "저작권" --format markdown > copyright.md
```

### 스크립트 연동

```bash
#!/bin/bash
# 법령 ID 추출
LAW_ID=$(warp law "민법" --format json | jq -r '.items[0].id')
echo "Law ID: $LAW_ID"

# 상세 정보 조회
warp law detail "$LAW_ID"
```

### 환경 변수 활용

```bash
# API 키를 환경 변수로
export LAW_API_KEY="your_api_key"
warp config set law.key $LAW_API_KEY

# 디버그 모드
export RUST_LOG=debug
warp law "민법" --verbose
```

### 별칭(Alias) 설정

```bash
# ~/.bashrc 또는 ~/.zshrc에 추가
alias wl='warp law'
alias wp='warp precedent'
alias ws='warp search'

# 사용
wl "민법"
wp "음주운전"
ws "개인정보"
```

### 자동완성 설정 (개발 예정)

```bash
# Bash
eval "$(warp completions bash)"

# Zsh
eval "$(warp completions zsh)"

# Fish
warp completions fish | source
```

## 팁과 트릭

### 1. 빠른 검색

```bash
# 최신 법령만
warp law "개인정보" --date-from $(date -d '30 days ago' +%Y%m%d)

# 첫 결과만
warp law "민법" --size 1
```

### 2. 결과 저장

```bash
# 여러 형식으로 저장
for fmt in json csv markdown html; do
    warp law "개인정보" --format $fmt > "result.$fmt"
done
```

### 3. 일괄 처리

```bash
# 여러 법령 검색
for law in "민법" "형법" "민사소송법"; do
    echo "=== $law ==="
    warp law "$law" --size 3
done
```

### 4. 모니터링

```bash
# 변경사항 감지
watch -n 3600 'warp law "개인정보" --date-from $(date +%Y%m%d)'
```

## 성능 최적화

### 결과 수 제한

```bash
# 필요한 만큼만 요청
warp law "민법" --size 10  # 빠름
warp law "민법" --size 100 # 느림
```

### 특정 소스만 검색

```bash
# 전체 검색 대신
warp search "개인정보" --source nlic  # 빠름
warp search "개인정보"                 # 느림
```

### 캐싱 활용 (개발 예정)

```bash
# 캐시 활성화
warp config set cache.enabled true
warp config set cache.ttl 3600
```

## 문제 해결

일반적인 문제는 [문제 해결 가이드](troubleshooting.md)를 참조하세요.

## 추가 자료

- [설치 가이드](installation.md)
- [사용 예제](examples.md)
- [FAQ](faq.md)
- [문제 해결](troubleshooting.md)
