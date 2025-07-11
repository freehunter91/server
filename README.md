# CDN 서버

이 프로젝트는 실시간 모니터링 기능을 갖춘 Rust 기반 CDN 서버입니다.

## 기능

- `/static` 디렉토리의 정적 파일을 루트 경로(`/`)에서 제공합니다.
- `/dashboard_assets` 디렉토리의 대시보드 자산을 `/dash` 경로에서 제공합니다.
- WebSocket을 통한 실시간 CPU, 메모리 및 네트워크 사용량 모니터링.
- 지난 30일간의 일별 평균 네트워크 전송률 그래프.
- `config.toml`을 통해 호스트, 포트, 로그 레벨을 설정할 수 있습니다.

## 로컬 개발

### 전제 조건

- Rust (`rustup` 포함)
- Node.js (`dashboard_assets/main.js`를 수정할 계획이라면 필요하지만, 실행에는 필수는 아님)

### 서버 실행

1.  **의존성 설치:**

    ```bash
    cargo build
    ```

2.  **서버 실행:**

    ```bash
    cargo run
    ```

    서버는 `http://0.0.0.0:8080`에서 시작됩니다 (`config.toml`에 설정된 경우).

3.  **대시보드 접속:**

    브라우저를 열고 `http://localhost:8080/dash`로 이동합니다.

    *대시보드가 올바르게 표시되지 않으면, 브라우저 캐시를 지우기 위해 강제 새로고침(Ctrl+Shift+R 또는 Cmd+Shift+R)을 시도해 보세요.*

## 설정

서버 설정은 `config.toml`에서 할 수 있습니다:

```toml
host = "0.0.0.0"
port = 8080
log_level = "info"
```

## Docker 배포 (NAS에서)

이 섹션에서는 NAS 장비에서 Docker 이미지를 빌드하고 실행하는 방법을 설명합니다.

### 전제 조건

-   NAS에 대한 SSH 접속.
-   NAS에 Docker가 설치되어 있고 실행 중이어야 합니다.

### 단계

1.  **프로젝트 파일을 NAS로 복사:**

    로컬 머신의 터미널에서 `scp`를 사용하여 전체 프로젝트 디렉토리를 NAS로 복사합니다. `your_nas_username`을 실제 NAS 사용자 이름으로, `192.168.35.15`를 NAS IP 주소로 바꾸세요.

    ```bash
    scp -r /media/gwkang/Projects/workspace/rust/cdn_v1 your_nas_username@192.168.35.15:/volume1/.@root/docker_files/
    ```

2.  **NAS에서 Docker 이미지 빌드:**

    파일이 NAS로 복사된 후, SSH를 통해 NAS에 접속하여 프로젝트 디렉토리로 이동합니다. 그런 다음, `cdn:1` 태그로 Docker 이미지를 빌드합니다.

    ```bash
    ssh your_nas_username@192.168.35.15
    cd /volume1/.@root/docker_files/cdn_v1
    docker build -t cdn:1 .
    ```

3.  **NAS에서 Docker 컨테이너 실행:**

    이미지가 빌드되면 컨테이너를 실행할 수 있습니다. `sysinfo` 라이브러리가 컨테이너 내부의 시스템 정보를 제대로 수집하려면 추가 권한이 필요할 수 있습니다. `--privileged` 플래그를 사용하여 컨테이너를 실행하는 것을 권장합니다.

    ```bash
    docker run -d -p 8080:8080 --name cdn_server_instance --privileged cdn:1
    ```

    *참고: NAS에서 8080 포트가 이미 사용 중이 아닌지 확인하세요. `--privileged` 모드는 컨테이너에 광범위한 권한을 부여하므로, 프로덕션 환경에서는 신중하게 사용해야 합니다.*

## 문제 해결

-   **`Address already in use` 오류:** 다른 프로세스가 서버 포트(기본 8080)를 사용 중입니다. 해당 프로세스를 찾아 종료하거나 `config.toml`에서 `port`를 변경하세요.
-   **대시보드 업데이트 안됨:** 브라우저 캐시를 지우기 위해 강제 새로고침(Ctrl+Shift+R 또는 Cmd+Shift+R)을 시도해 보세요.
-   **`rustc` 버전 문제:** `rustc` 버전과 관련된 컴파일 오류가 발생하면 Rust 툴체인을 업데이트해 보세요:
    ```bash
    rustup update
    ```
-   **Docker 컨테이너 내 모니터링 데이터 누락:**
    -   **원인:** `sysinfo` 라이브러리가 컨테이너 내부에서 시스템 정보에 접근하기 위한 권한이 부족하거나, 웹소켓 연결 주소 문제일 수 있습니다.
    -   **해결:**
        1.  **컨테이너 권한:** 컨테이너 실행 시 `--privileged` 플래그를 사용하거나 `--cap-add SYS_PTRACE` 옵션을 추가하여 `sysinfo`가 시스템 정보에 접근할 수 있도록 합니다. (위 Docker 실행 명령에 반영됨)
        2.  **웹소켓 연결 주소:** 대시보드(`dashboard_assets/main.js`)에서 웹소켓 연결 주소를 `ws://localhost:8080/ws`와 같이 명시적으로 지정해야 합니다. `https://cdn.haja.run`과 같은 HTTPS 주소로 대시보드에 접속하는 경우, 브라우저의 보안 정책으로 인해 `ws://` 연결이 차단될 수 있습니다. 이 경우 리버스 프록시/CDN에서 `wss://`로 WebSocket 연결을 프록시하도록 설정해야 합니다.

