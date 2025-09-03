# 사용 예제

실제 시나리오별 Warp CLI 사용 예제를 소개합니다.

## 🚀 병렬 검색 (고성능 모드)

새로 추가된 병렬 검색 기능을 사용하면 3-5배 빠른 검색이 가능합니다.

### 기본 병렬 검색

```bash
# 모든 API에서 병렬 검색 (기본 3개 API: NLIC, ELIS, PREC)
warp search --parallel "민법"

# 특정 API들만 선택하여 병렬 검색
warp search --parallel --apis nlic,elis,prec "계약"
warp search --parallel --apis nlic,admrul "행정규칙"
```

### 고급 최적화 옵션

```bash
# 배치 처리로 성능 향상 (요청을 그룹화)
warp search --parallel --batch "부동산"

# 배치 크기 조정 (기본 10개)
warp search --parallel --batch --batch-size 20 "임대차"

# 캐싱 활성화 (1=기본, 2=고급)
warp search --parallel --cache-tier 2 "상속"

# 동시 연결 수 조정 (기본 5개)
warp search --parallel --max-concurrent 10 "민사소송"

# 모든 최적화 옵션 활성화
warp search --parallel --batch --cache-tier 2 --max-concurrent 8 "개인정보보호"
```

### 성능 비교

```bash
# 일반 검색 (순차 처리)
time warp search "민법" --source all

# 병렬 검색 (3-5배 빠름)
time warp search --parallel "민법"

# 최적화된 병렬 검색 (최고 성능)
time warp search --parallel --batch --cache-tier 2 "민법"
```

## 📊 성능 메트릭스 모니터링

새로 추가된 메트릭스 시스템으로 Warp CLI의 성능을 실시간으로 모니터링할 수
있습니다.

### 실시간 성능 대시보드

```bash
# 기본 성능 대시보드 표시
warp metrics show

# 상세한 성능 분석 포함
warp metrics show --details

# 실시간 모니터링 (5초마다 갱신)
warp metrics show --refresh 5s

# 1분 간격 모니터링
warp metrics show --refresh 1m

# 특정 시간 범위의 메트릭스
warp metrics show --window 1h --details
```

### 캐시 및 연결 풀 모니터링

```bash
# 캐시 성능 확인
warp metrics cache

# 연결 풀 상태 확인
warp metrics pools

# 지연시간 분석 (백분위수 포함)
warp metrics latency --percentiles 50,90,95,99
```

### 히스토리 및 리포트

```bash
# 최근 24시간 성능 히스토리
warp metrics history --hours 24

# 최근 7일 성능 트렌드
warp metrics history --days 7

# 특정 API 성능 분석
warp metrics history --api nlic --days 3

# JSON 형식 성능 리포트
warp metrics report --format json

# CSV 형식으로 내보내기
warp metrics report --format csv > performance_report.csv
```

### 메트릭스 관리

```bash
# 메트릭스 데이터 초기화
warp metrics reset --force

# 30일 이전 데이터 정리
warp metrics cleanup --older-than 30

# 메트릭스 수집 비활성화
warp metrics disable
```

### 예상 출력 예시

```
📊 Performance Dashboard
────────────────────────────────────────────────────────────
🖥️  System Overview
   Uptime: 2h 15m 30s
   Memory: 45.2 MB
   Window: Last5Minutes

🔄 API Operations
   Total Requests: 1,247
   Success Rate: 98.4% ✅
   Top Operations:
     • nlic_search (456 req, 201ms avg, 99.1%)
     • elis_search (321 req, 289ms avg, 97.8%)
     • prec_search (234 req, 312ms avg, 96.2%)

💾 Cache Performance
   nlic: 72.1% hit rate (328 hits, 128 misses)
     Storage: 12.4 MB (1,247 entries)
   elis: 61.4% hit rate (197 hits, 124 misses)
     Storage: 8.7 MB (891 entries)

🔗 Connection Pools
   nlic_pool: 40.0% utilization (8/20 active)
   elis_pool: 25.0% utilization (5/20 active)
```

## 📚 기본 사용 예제

### 법령 검색 및 조회

#### 간단한 법령 검색

```bash
# 민법 검색
warp law "민법"

# 결과 개수 제한
warp law "개인정보보호법" --size 5

# 특정 페이지 조회
warp law "도로교통법" --page 2 --size 10
```

#### 법령 상세 정보 조회

```bash
# 1. 먼저 법령 검색으로 ID 확인
warp law "민법" --format json | jq '.items[0]'

# 2. ID로 상세 정보 조회
warp law detail MST000001

# 3. 법령 개정 이력 확인
warp law history MST000001
```

### 판례 검색

#### 대법원 판례 검색

```bash
# 기본 판례 검색
warp precedent "음주운전"

# 법원 지정
warp precedent "횡령" --court "대법원"

# 특정 기간 판례
warp precedent "개인정보" --date-from 20230101 --date-to 20231231
```

#### 판례 상세 정보

```bash
# 판례 상세 조회
warp precedent detail CASE_ID

# JSON 형식으로 저장
warp precedent "저작권" --format json > copyright_cases.json
```

### 자치법규 검색

#### 지역별 자치법규

```bash
# 서울시 자치법규
warp ordinance "서울"

# 특정 구 조례
warp ordinance "강남구 주차"

# 조례 상세 정보
warp ordinance detail ORD_ID
```

### 행정규칙 검색

```bash
# 행정안전부 규칙
warp admrule "개인정보" --department "행정안전부"

# 모든 부처 검색
warp admrule "정보공개"
```

### 법령해석례 검색

```bash
# 건축법 해석례
warp interpretation "건축법"

# 결과 제한
warp interpretation "세법" --size 3
```

## 🔄 통합 검색

### 모든 소스에서 검색

```bash
# 전체 검색
warp search "개인정보보호"

# 출력 형식 지정
warp search "저작권" --format markdown

# CSV로 저장
warp search "부동산" --format csv > real_estate_laws.csv
```

### 특정 소스 조합

```bash
# 법령과 판례만 검색
warp search "음주운전" --source nlic,prec

# 법령, 자치법규, 행정규칙 검색
warp search "환경" --source nlic,elis,admrul
```

## 🎯 고급 사용 시나리오

### 시나리오 1: 법령 개정 이력 추적

```bash
#!/bin/bash
# 법령 개정 이력 추적 스크립트

QUERY="개인정보보호법"

echo "=== $QUERY 개정 이력 추적 ==="

# 1. 법령 검색
LAW_JSON=$(warp law "$QUERY" --format json)
LAW_ID=$(echo "$LAW_JSON" | jq -r '.items[0].id')
LAW_TITLE=$(echo "$LAW_JSON" | jq -r '.items[0].title')

echo "법령: $LAW_TITLE (ID: $LAW_ID)"

# 2. 개정 이력 조회
echo -e "\n개정 이력:"
warp law history "$LAW_ID" --format markdown

# 3. 최근 개정 내용 저장
warp law history "$LAW_ID" --format json > "${QUERY}_history.json"
echo -e "\n개정 이력이 ${QUERY}_history.json 에 저장되었습니다."
```

### 시나리오 2: 특정 기간 법령 변경사항 모니터링

```bash
#!/bin/bash
# 월간 법령 변경사항 리포트

YEAR=2024
MONTH=10

echo "=== ${YEAR}년 ${MONTH}월 법령 변경사항 ==="

# 날짜 범위 설정
START_DATE="${YEAR}${MONTH}01"
END_DATE="${YEAR}${MONTH}31"

# 주요 법령 목록
LAWS=("민법" "형법" "개인정보보호법" "도로교통법" "건축법")

for LAW in "${LAWS[@]}"; do
    echo -e "\n[$LAW]"
    warp law "$LAW" \
        --date-from "$START_DATE" \
        --date-to "$END_DATE" \
        --format table
done

# 결과를 마크다운 파일로 저장
{
    echo "# ${YEAR}년 ${MONTH}월 법령 변경사항 리포트"
    echo ""
    for LAW in "${LAWS[@]}"; do
        echo "## $LAW"
        warp law "$LAW" \
            --date-from "$START_DATE" \
            --date-to "$END_DATE" \
            --format markdown
        echo ""
    done
} > "monthly_report_${YEAR}_${MONTH}.md"

echo -e "\n리포트가 monthly_report_${YEAR}_${MONTH}.md 에 저장되었습니다."
```

### 시나리오 3: 판례 분석 자동화

```bash
#!/bin/bash
# 특정 주제 판례 수집 및 분석

TOPIC="개인정보 침해"
OUTPUT_DIR="case_analysis"

mkdir -p "$OUTPUT_DIR"

echo "=== '$TOPIC' 관련 판례 분석 ==="

# 1. 최근 1년 판례 검색
CURRENT_YEAR=$(date +%Y)
LAST_YEAR=$((CURRENT_YEAR - 1))
START_DATE="${LAST_YEAR}0101"
END_DATE="${CURRENT_YEAR}1231"

# 2. 판례 데이터 수집
echo "판례 수집 중..."
warp precedent "$TOPIC" \
    --date-from "$START_DATE" \
    --date-to "$END_DATE" \
    --format json > "$OUTPUT_DIR/cases.json"

# 3. 법원별 통계
echo -e "\n법원별 판례 수:"
jq '.items | group_by(.court) | map({court: .[0].court, count: length})' \
    "$OUTPUT_DIR/cases.json"

# 4. 월별 판례 수 통계
echo -e "\n월별 판례 수:"
jq '.items | group_by(.date[0:6]) | map({month: .[0].date[0:6], count: length})' \
    "$OUTPUT_DIR/cases.json"

# 5. HTML 리포트 생성
warp precedent "$TOPIC" \
    --date-from "$START_DATE" \
    --date-to "$END_DATE" \
    --format html > "$OUTPUT_DIR/report.html"

echo -e "\n분석 결과가 $OUTPUT_DIR 디렉토리에 저장되었습니다."
```

### 시나리오 4: 자치법규 비교 분석

```bash
#!/bin/bash
# 여러 지역의 유사 조례 비교

KEYWORD="주차장"
REGIONS=("서울" "부산" "대구" "인천" "광주")

echo "=== 지역별 '$KEYWORD' 조례 비교 ==="

for REGION in "${REGIONS[@]}"; do
    echo -e "\n[$REGION]"
    warp ordinance "$REGION $KEYWORD" --size 3 --format table
done

# CSV로 통합 저장
{
    echo "지역,제목,제정일"
    for REGION in "${REGIONS[@]}"; do
        warp ordinance "$REGION $KEYWORD" --size 5 --format csv | tail -n +2
    done
} > "regional_comparison_${KEYWORD}.csv"

echo -e "\n비교 결과가 regional_comparison_${KEYWORD}.csv 에 저장되었습니다."
```

## 🤖 자동화 및 스크립팅

### Python 연동 예제

```python
#!/usr/bin/env python3
import subprocess
import json
from datetime import datetime, timedelta

def search_laws(query, days_back=30):
    """최근 N일간 개정된 법령 검색"""

    # 날짜 계산
    end_date = datetime.now()
    start_date = end_date - timedelta(days=days_back)

    # 명령어 실행
    cmd = [
        'warp', 'law', query,
        '--date-from', start_date.strftime('%Y%m%d'),
        '--date-to', end_date.strftime('%Y%m%d'),
        '--format', 'json'
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode == 0:
        data = json.loads(result.stdout)
        return data['items']
    else:
        print(f"Error: {result.stderr}")
        return []

def main():
    # 최근 30일간 개정된 개인정보 관련 법령
    laws = search_laws("개인정보", days_back=30)

    print(f"최근 30일간 개정된 법령: {len(laws)}건")

    for law in laws:
        print(f"- {law['title']} ({law['date']})")

    # 결과를 JSON 파일로 저장
    with open('recent_laws.json', 'w', encoding='utf-8') as f:
        json.dump(laws, f, ensure_ascii=False, indent=2)

if __name__ == "__main__":
    main()
```

### Bash 함수로 활용

```bash
# ~/.bashrc 또는 ~/.zshrc에 추가

# 법령 빠른 검색
function law() {
    warp law "$1" --size 5 --format table
}

# 판례 빠른 검색
function case() {
    warp precedent "$1" --size 5 --format table
}

# 통합 검색 후 저장
function law_save() {
    local query="$1"
    local filename="${2:-${query// /_}.json}"
    warp search "$query" --format json > "$filename"
    echo "Results saved to $filename"
}

# 법령 상세 보기 (브라우저에서 열기)
function law_detail() {
    local law_id="$1"
    warp law detail "$law_id" --format html > /tmp/law_detail.html
    open /tmp/law_detail.html  # macOS
    # xdg-open /tmp/law_detail.html  # Linux
}
```

### CI/CD 통합 예제

```yaml
# .github/workflows/law-check.yml
name: Legal Compliance Check

on:
  schedule:
    - cron: "0 9 * * 1" # 매주 월요일 9시
  workflow_dispatch:

jobs:
  check-laws:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Warp CLI
        run: cargo install warp

      - name: Configure API Key
        run: |
          warp config init
          warp config set law.key ${{ secrets.LAW_API_KEY }}

      - name: Check Privacy Laws
        run: |
          warp law "개인정보보호법" --format json > privacy_laws.json

      - name: Check Recent Changes
        run: |
          warp law "개인정보" \
            --date-from $(date -d '7 days ago' +%Y%m%d) \
            --date-to $(date +%Y%m%d) \
            --format markdown > recent_changes.md

      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: law-reports
          path: |
            privacy_laws.json
            recent_changes.md
```

### 크론탭 설정 예제

```bash
# 매일 오전 9시 법령 업데이트 확인
0 9 * * * /usr/local/bin/warp law "개인정보보호법" --format json > /var/log/laws/daily_$(date +\%Y\%m\%d).json

# 매주 월요일 주간 리포트 생성
0 10 * * 1 /home/user/scripts/weekly_law_report.sh

# 매월 1일 월간 통계 생성
0 0 1 * * /usr/local/bin/warp search "세법" --format csv > /var/reports/monthly_$(date +\%Y\%m).csv
```

## 📊 데이터 분석 예제

### jq를 활용한 JSON 처리

```bash
# 제목만 추출
warp law "민법" --format json | jq -r '.items[].title'

# 특정 날짜 이후 법령만 필터
warp law "개인정보" --format json | \
    jq '.items[] | select(.date >= "20240101")'

# 통계 생성
warp search "환경" --format json | \
    jq '{
        total: .total,
        by_source: .items | group_by(.source) |
        map({source: .[0].source, count: length})
    }'
```

### CSV 데이터 처리

```bash
# CSV를 Excel로 변환 (Python pandas 필요)
python -c "
import pandas as pd
df = pd.read_csv('laws.csv')
df.to_excel('laws.xlsx', index=False)
"

# 특정 컬럼만 추출
warp law "민법" --format csv | cut -d',' -f1,2,4

# 정렬 및 중복 제거
warp law "세법" --format csv | sort -u
```

## 📚 관련 문서

- [설치 가이드](installation.md)
- [사용자 가이드](user-guide.md)
- [문제 해결](troubleshooting.md)
- [FAQ](faq.md)
