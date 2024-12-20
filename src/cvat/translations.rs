use once_cell::sync::Lazy;
use std::collections::HashMap;
use serde_json::{Value, json};
use std::error::Error;

#[derive(Debug)]
struct ErrorInfo {
    code: i64,
    patterns: Vec<&'static str>,
    messages: Vec<&'static str>,
}

static ERROR_TRANSLATIONS: Lazy<HashMap<i64, ErrorInfo>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0, ErrorInfo { 
        code: 0, 
        patterns: vec!["正常退出"],
        messages: vec!["정상 종료"]
    });
    m.insert(10, ErrorInfo { 
        code: 10, 
        patterns: vec!["无效句柄或指定句柄所指向窗口不存在"],
        messages: vec!["잘못된 핸들, 또는 지정된 핸들이 가리키는 창이 존재하지 않습니다."]
    });
    m.insert(101, ErrorInfo { 
        code: 101, 
        patterns: vec!["读取图片失败，图片为空", "未能找到原神窗口句柄"],
        messages: vec!["이미지 읽기 실패, 이미지가 비어있음", "원신 창 핸들을 찾을 수 없음"]
    });
    m.insert(103, ErrorInfo {
        code: 103,
        patterns: vec!["获取原神画面失败"],
        messages: vec!["원신 화면 가져오기 실패"]
    });
    m.insert(1001, ErrorInfo { 
        code: 1001, 
        patterns: vec!["获取所有信息时，没有识别到paimon", "获取坐标时，没有识别到paimon"],
        messages: vec!["모든 정보를 가져올 때 페이몬 아이콘을 인식하지 못함", "좌표를 가져올 때 페이몬 아이콘을 인식하지 못함"]
    });
    m.insert(10003, ErrorInfo { 
        code: 10003, 
        patterns: vec!["句柄为空", "获取下一帧画面失败"],
        messages: vec!["핸들이 비어있음", "다음 프레임 가져오기 실패"]
    });
    m.insert(10001, ErrorInfo {
        code: 10001,
        patterns: vec!["重新初始化捕获池"],
        messages: vec!["캡처 풀 재초기화"]
    });
    m.insert(10002, ErrorInfo {
        code: 10002,
        patterns: vec!["获取新的画面失败"],
        messages: vec!["새 화면 가져오기 실패"]
    });
    m.insert(10004, ErrorInfo {
        code: 10004,
        patterns: vec!["未能获取到新一帧画面"],
        messages: vec!["새 프레임을 가져올 수 없음"]
    });
    m.insert(10005, ErrorInfo {
        code: 10005,
        patterns: vec!["未能从GPU拷贝画面到CPU"],
        messages: vec!["GPU에서 CPU로 화면 복사 실패"]
    });
    m.insert(10006, ErrorInfo {
        code: 10006,
        patterns: vec!["设置的句柄为空", "指针指向为空"],
        messages: vec!["설정된 핸들이 비어있음", "포인터가 비어있음"]
    });
    m.insert(10013, ErrorInfo {
        code: 10013,
        patterns: vec!["未能捕获窗口"],
        messages: vec!["창을 캡처할 수 없음"]
    });
    m.insert(11, ErrorInfo { 
        code: 11, 
        patterns: vec!["无效句柄或指定句柄所指向窗口不存在", "原神角色小箭头区域为空"],
        messages: vec!["유효하지 않은 핸들 또는 지정된 창이 존재하지 않음", "원신 캐릭터 화살표 영역이 비어있음"]
    });
    m.insert(12, ErrorInfo {
        code: 12,
        patterns: vec!["窗口句柄失效"],
        messages: vec!["창 핸들이 유효하지 않음"]
    });
    m.insert(14, ErrorInfo { 
        code: 14, 
        patterns: vec!["窗口画面大小小于480x360，无法使用", "窗口画面小于裁剪框，截图失败"],
        messages: vec!["창 크기가 480x360보다 작아 사용할 수 없음", "창이 자르기 영역보다 작아 스크린샷 실패"]
    });
    m.insert(2001, ErrorInfo {
        code: 2001,
        patterns: vec!["获取角色朝向时，没有识别到paimon"],
        messages: vec!["캐릭터 방향을 가져올 때 페이몬 아이콘을 인식하지 못함"]
    });
    m.insert(251, ErrorInfo {
        code: 251,
        patterns: vec!["路径缓存区为空指针或是路径缓存区大小为小于1"],
        messages: vec!["경로 버퍼가 비어있거나 크기가 1보다 작음"]
    });
    m.insert(252, ErrorInfo {
        code: 252,
        patterns: vec!["画面为空", "保存画面失败，请检查文件路径是否合法"],
        messages: vec!["화면이 비어있음", "화면 저장 실패, 파일 경로가 올바른지 확인하세요"]
    });
    m.insert(291, ErrorInfo {
        code: 291,
        patterns: vec!["缓存区为空指针或是缓存区大小为小于1"],
        messages: vec!["버퍼가 비어있거나 크기가 1보다 작음"]
    });
    m.insert(292, ErrorInfo {
        code: 292,
        patterns: vec!["缓存区大小不足"],
        messages: vec!["버퍼 크기 부족"]
    });
    m.insert(3, ErrorInfo {
        code: 3,
        patterns: vec!["窗口画面为空"],
        messages: vec!["창 화면이 비어있음"]
    });
    m.insert(3001, ErrorInfo {
        code: 3001,
        patterns: vec!["获取视角朝向时，没有识别到paimon"],
        messages: vec!["시점 방향을 가져올 때 페이몬 아이콘을 인식하지 못함"]
    });
    m.insert(3006, ErrorInfo {
        code: 3006,
        patterns: vec!["获取视角的误差过大"],
        messages: vec!["시점 오차가 너무 큼"]
    });
    m.insert(4001, ErrorInfo {
        code: 4001,
        patterns: vec!["获取神瞳时，没有识别到paimon"],
        messages: vec!["신의 눈을 가져올 때 페이몬 아이콘을 인식하지 못함"]
    });
    m.insert(40101, ErrorInfo { 
        code: 40101, 
        patterns: vec!["Bitblt模式下检测派蒙失败"],
        messages: vec!["Bitblt 모드에서 페이몬 아이콘 감지 실패"]
    });
    m.insert(40102, ErrorInfo {
        code: 40102,
        patterns: vec!["Bitblt模式下没有检测到派蒙"],
        messages: vec!["Bitblt 모드에서 페이몬 아이콘이 감지되지 않음"]
    });
    m.insert(40105, ErrorInfo {
        code: 40105,
        patterns: vec!["Bitblt模式下计算小地图区域失败"],
        messages: vec!["Bitblt 모드에서 미니맵 영역 계산 실패"]
    });
    m.insert(40201, ErrorInfo { 
        code: 40201, 
        patterns: vec!["DirectX模式下检测派蒙失败"],
        messages: vec!["DirectX 모드에서 페이몬 아이콘 감지 실패"]
    });
    m.insert(40202, ErrorInfo {
        code: 40202,
        patterns: vec!["DirectX模式下没有检测到派蒙"],
        messages: vec!["DirectX 모드에서 페이몬 아이콘이 감지되지 않음"]
    });
    m.insert(40205, ErrorInfo {
        code: 40205,
        patterns: vec!["DirectX模式下计算小地图区域失败"],
        messages: vec!["DirectX 모드에서 미니맵 영역 계산 실패"]
    });
    m.insert(433, ErrorInfo {
        code: 433,
        patterns: vec!["截图失败"],
        messages: vec!["스크린샷 실패"]
    });
    m.insert(5, ErrorInfo {
        code: 5,
        patterns: vec!["原神小地图区域为空"],
        messages: vec!["원신 미니맵 영역이 비어있음"]
    });
    m.insert(601, ErrorInfo {
        code: 601,
        patterns: vec!["获取神瞳失败，未确定原因"],
        messages: vec!["신의 눈 가져오기 실패, 원인 불명"]
    });
    m.insert(8, ErrorInfo {
        code: 8,
        patterns: vec!["未能在UID区域检测到有效UID"],
        messages: vec!["UID 영역에서 유효한 UID를 감지하지 못함"]
    });
    m.insert(9, ErrorInfo {
        code: 9,
        patterns: vec!["提取小箭头特征误差过大"],
        messages: vec!["작은 화살표 특징 추출 오차가 너무 큼"]
    });
    m.insert(9001, ErrorInfo {
        code: 9001,
        patterns: vec!["传入图片通道不对应"],
        messages: vec!["입력된 이미지 채널이 일치하지 않음"]
    });
    m.insert(9002, ErrorInfo {
        code: 9002,
        patterns: vec!["传入图片为空"],
        messages: vec!["입력된 이미지가 비어있음"]
    });
    m
});

pub fn translate_error_json(error_json: &str) -> Result<String, Box<dyn Error>> {
    let v: Value = serde_json::from_str(error_json)?;
    
    if let Some(error_list) = v["errorList"].as_array() {
        let translated_errors: Vec<Value> = error_list
            .iter()
            .map(|error| {
                let code = error["code"].as_i64().unwrap_or(0);
                let msg = error["msg"].as_str().unwrap_or("");
                
                let translated_msg = ERROR_TRANSLATIONS
                    .get(&code)
                    .and_then(|info| {
                        info.patterns.iter()
                            .position(|pattern| msg.contains(pattern))
                            .map(|index| info.messages[index])
                    })
                    .unwrap_or_else(|| msg);
                
                json!({
                    "code": code,
                    "msg": translated_msg
                })
            })
            .collect();
            
        Ok(json!({ "errorList": translated_errors }).to_string())
    } else {
        Ok(error_json.to_string())
    }
} 