# Genshin-Paisitioning-App

원신 게임의 미니맵을 실시간으로 캡처하여 브라우저 상의 지도에 표시하는 프로젝트입니다.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/W7W2UWJ60)

## 프로젝트 소개

**Genshin-Paisitioning 프로젝트는**
**실시간으로 원신 화면을 캡쳐해 미니맵을 이미지 인식해서**
**현재 위치를 게임닷 맵스 사이트 화면에 띄워주는 프로그램/스크립트입니다.**

이 프로젝트는 크게 두 가지로 구성됩니다:

1. **Genshin Paisitioning App** (줄여서 GPA)
2. **Genshin Paisitioning Script** (줄여서 GPS)

## Genshin Paisitioning App (GPA)

원신 게임 화면을 캡쳐하고 위치 정보를 웹 페이지로 전송하기 위한 런타임 프로그램입니다.

### 🚀 설치 방법

1. [링크](https://github.com/Haytsir/Genshin-Paisitioning-App/releases/latest)에서 `genshin_paisitioning_app.zip`을 다운로드 받습니다.
2. zip파일 내부의 `genshin_paisitioning_app.exe`를 실행합니다.
3. 실행하면 관리자 권한을 요구합니다, 수락합니다.
4. 설치 완료 (설치 위치 `%localappdata%\genshin-paisitioning`)

⚠️ **주의**: zip 파일 안의 exe를 `%localappdata%\genshin-paisitioning` 경로에 넣으라는 것이 아닙니다!
`genshin_paisitioning_app.exe`는 **실행된 경로가 위 경로가 아닐때에만** 설치 과정을 진행합니다.

### 🗑️ 삭제 방법

1. 제어판 - 프로그램 제거 - [원신 파이지셔닝] 우클릭 - [제거/변경(U)]
2. (윈11 이라면, 또는) 설정 - 앱 - 설치된 앱 - [원신 파이지셔닝] 검색 - […] 선택 - [제거]
3. [윈도우 키]+[R] 키 - `%localappdata%` 입력 - genshin-paisitioning 폴더 찾아보고, 있다면 삭제 (없을 수도 있음)

## Genshin Paisitioning Script (GPS)

브라우저에서 GPA와 통신하고, 전달받은 인게임 위치를 맵스 화면에 찍어주기 위한 유저스크립트입니다.

## 사용 방법

1. **게임 내 설정 변경**

   - 게임 내 [설정] - [기타] - [미니맵 고정: 방향 고정]으로 설정
2. **게임닷 맵스 열기**

   - [게임닷 맵스](https://genshin.gamedot.org/?mid=genshinmaps) 열면 실시간 연결 버튼이 생김
3. **실시간 연결**

   - 이 버튼을 누르면 GPA를 실행하고, GPS가 GPA를 준비된 상태로 만들기 위해 업데이트 버전 등등을 점검하고 GPA에게 원신 화면을 인식 시작하라는 명령을 보냅니다.

**다시 말해서, GPA(genshin-paisitioning.exe)는 단독 실행 할 수 없고,**
**GPS(브라우저 측)로부터 실행 요청이 정상적으로 이루어져야 실행 가능합니다.**

## 동작 구조

```
cvAutoTrack ==라이브러리로 호출됨=> GPA <= WebSocket으로 상호 통신 => GPS <= UI로 상호 작용 => 브라우저
```

## QnA

1. 획득한 상자같은 것들도 자동으로 감지되나요?

- 아니요

2. 안전한가요? 정지 위험은?

- 이 프로젝트는 비공식 프로젝트로 사용함에 있어서의 책임은 사용자에게 있습니다.

3. GPA는 어떻게 끄나요?
- GPA는 GPS와 연결된 상태였다면, GPS와 연결이 끊기는 경우(브라우저 탭을 닫는 등) 자동으로 종료됩니다. 하지만 연결이 안된 상태에서 직접 종료하려면 시스템 트레이에서 종료할 수 있습니다.

   ![](https://raw.githubusercontent.com/Haytsir/Genshin-Paisitioning-App/refs/heads/master/docs/images/01.png)

4. 작동이 안되는 것 같아요! (GPA-GPS 연결 자체가 안될 경우)

- GPA를 수동으로 종료한 뒤, 게임닷 맵스를 새로고침하고 다시 [실시간 연결]을 시도해보세요
  수 차례 해도 똑같은 상황이라면 깃허브 Issue 등록시 자세한 상황을 말해주면 검토해보겠습니다. 또는 자바 스크립트 또는 브라우저 콘솔을 다룰 줄 아는 사람이라면 [실시간 연결] 버튼을 우측 마우스 클릭하면 GPS가 debug 모드로 진입하니 문제 내용과 함께 WebSocket 통신 과정에서 받은 Object  내용을 펼쳐 보여준다면 더 도움이 됩니다.

5. 위치를 얻는 도중 오류가 발생했다고 계속 떠요

- GPA-GPS 서로 연결은 됐지만
GPA가 사용하는 cvAutoTrack이 원신 화면을 캡쳐하지 못했거나, 원신 화면은 캡쳐했는데 미니맵을 읽지 못했거나, 미니맵에서 위치, 방향 등을 찾아내지 못한 경우입니다. 사용자에 따라서 여러 원인이 있을 수 있는데, 주된 이유는 Nvidia 또는 Amd 그래픽 필터 또는 외부 그래픽 옵션 사용, 프레임/리소스 출력 프로그램이 미니맵이나 페이몬(메뉴) 아이콘을 가림, 미니맵이 방향 고정돼있지 않음 등이 대표적인 원인입니다.

6. genshin_paisitioning_app.exe 실행시 설치 완료 다이얼로그가 나타나지 않았다면

   ![](https://raw.githubusercontent.com/Haytsir/Genshin-Paisitioning-App/refs/heads/master/docs/images/02.png)

- 시작 - cmd 검색 - 관리자 권한으로 실행
   `%localappdata%\genshin-paisitioning\genshin_paisitioning_app.exe --install`
   복사 후 붙여넣어서 다이얼로그가 나타나는지 확인해보세요

7. 처음 위치는 잡는데 그 뒤로 마커가 움직이질 않아요
- 원신 게임 실행 파일(.exe)를 찾은 뒤, 실행 파일을 우클릭하여 속성 - 호환성 - 전체 화면 최적화 해제 체크 - 적용
- NVIDIA 또는 AMD 소프트웨어에서 설정할 수도 있습니다.

8. 작동은 되는데 화살표 마커가 뚝뚝 끊겨 움직이고 막 날아다녀요
- 게임 화면을 캡쳐해서 캡쳐된 이미지를 토대로 위치를 특정하는 방식이기 때문에 브라우저에 표시된 위치가 실제 게임 위치와 다르거나 인식을 잘 못하는 지역에 있다면 순간이동처럼 막 이상한 데 찍는 경우가 있습니다. 이는 왠만해선 컴퓨터의 리소스가 부족하거나 하는 문제가 아닙니다.

## 테스트 환경

- ✅ **Chrome**: 의도한 방식으로 작동함
- ❓ **Firefox, Edge, Safari**: 해당 환경에서 테스트하지 않음, 동작은 할 것이지만 처리되지 못한 버그 발생 가능

## 최근 변경점

### 24-12-25

- **GPA v1.2.2**: 실행 초기에 딜레이 관련 설정이 적용되지 않던 문제 수정
- **GPS v1.7.3**: 위치 감지를 못하고 있는 상황에 0, 0 좌표로 이동하던 문제 수정

<details><summary>이전 버전 변경점</summary>

### 24-12-23

💻GPA v1.2.1

- 비동기 처리로 인해 업데이트 확인 과정에서 프로세스가 데드락 상태에 빠지는 문제 수정

### 24-12-21

💻GPA v1.2.0

- 자동 업데이트가 진행되지 않으면 수동 설치바람
- 버전 캐시를 24시간->2시간으로 변경
- 코드베이스가 너무 복잡해져서 개발 용이성을 확보하기 위해 대부분의 내부 로직을 변경함
- 눈에 띄지않는 성능 개선이 있을 것으로 기대함
- GPS에서 설정을 변경했을 때 반영되지 않는 문제 해결

### 24-12-03

💻GPA v1.1.15

- 짧은 시간 내에 여러 번 재 실행시 새 버전 탐지로 인한 rate limiting 방지를 위해 fetch 결과에 cache 적용.
- 이제 새 버전이 나타나도 자동 업데이트 로직에서 새 버전 인식은 cache 기간(24시간)이 지나야 적용됨.

### 24-11-25

💻GPA v1.1.12

- 설치시 설치 성공했다는 다이얼로그가 표시되지 않는 문제 수정,
- 설치 실패 시 다이얼로그 문자를 좀 더 다양하게 출력하여 문제점을 찾기 쉽도록 변경함
- GPA 자동 업데이트 방식 수정 및 제대로 동작하지 않는 문제 수정 (이전 버전을 사용하고 있다면 다시 설치하거나, 수동으로 파일을 집어넣어야 함)
- 기존 GPA 삭제시 발생할 수 있었던 잠재적 문제 완화
- cvAutoTrack 자동 업데이트 방식 수정
- cvAutoTrack의 버전 체계가 바뀜에 따라 단순히 버전 넘버 비교로 신버전을 알 수 없으므로, 일시적으로나마 버전 비교 방식을 파일의 마지막 수정 시간으로 함. 따라서 cvAutoTrack 신버전이 나왔더라도 그 이후에 수정 시간이 변경된 구버전을 집어넣을 경우 GPA가 이를 신버전으로 인식할 수도 있음.
- config-rs 라이브러리에서 발생하는 문제 해결, AppConfig struct에 대한 키 네임 스타일링 변경
- !해당 버전 사용 시 GPA 실행이 되지 않는 것 같으면 genshin-paisitioning/logs폴더에서 errors.log 확인, auto app update missing같은 에러 로그가 있다면 genshin-paisitioning/config.json 파일 삭제 후 재시도해볼 것

### 23-04-24
💻GPA v1.1.5
cvAutoTrack가 없는 상태에서 업데이트를 시도할 시 동작이 중단되는 버그 수정.

debug 모드로 실행시, 로그 파일을 생성하도록 변경.

💻GPA v1.1.4

릴리즈



</details>

## 알려진 문제점

## 기여하기

GPS 또는 GPA 기능 제안, 코드 수정(Pull Request), 문제(Issue) 등을 내는 것은 언제나 환영입니다.
Pull Request나 Issue 작성은 각각 GPS/GPA 깃허브에 해주시기 바랍니다.

## 라이선스

이 프로젝트는 오픈소스로 제공되며, 자세한 라이선스 정보는 각 저장소를 참조하시기 바랍니다.
