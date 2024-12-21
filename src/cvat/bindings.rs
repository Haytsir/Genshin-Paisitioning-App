/* automatically generated by rust-bindgen 0.69.1 */
#![allow( non_upper_case_globals
    , non_camel_case_types
    , non_snake_case
    , dead_code)]

#[derive(Debug)]
pub struct cvAutoTrack {
    __library: ::libloading::Library,
    pub verison: Result<
        unsafe extern "C" fn(versionBuff: *mut ::std::os::raw::c_char) -> bool,
        ::libloading::Error,
    >,
    pub init: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub uninit: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub startServe: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub stopServe: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub SetUseBitbltCaptureMode: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub SetUseDx11CaptureMode: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub SetHandle: Result<
        unsafe extern "C" fn(handle: ::std::os::raw::c_longlong) -> bool,
        ::libloading::Error,
    >,
    pub SetWorldCenter: Result<unsafe extern "C" fn(x: f64, y: f64) -> bool, ::libloading::Error>,
    pub SetWorldScale: Result<unsafe extern "C" fn(scale: f64) -> bool, ::libloading::Error>,
    pub ImportMapBlock: Result<
        unsafe extern "C" fn(
            id_x: ::std::os::raw::c_int,
            id_y: ::std::os::raw::c_int,
            image_data: *const ::std::os::raw::c_char,
            image_data_size: ::std::os::raw::c_int,
            image_width: ::std::os::raw::c_int,
            image_height: ::std::os::raw::c_int,
        ) -> bool,
        ::libloading::Error,
    >,
    pub ImportMapBlockCenter: Result<
        unsafe extern "C" fn(x: ::std::os::raw::c_int, y: ::std::os::raw::c_int) -> bool,
        ::libloading::Error,
    >,
    pub ImportMapBlockCenterScale: Result<
        unsafe extern "C" fn(
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            scale: f64,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetTransformOfMap: Result<
        unsafe extern "C" fn(
            x: *mut f64,
            y: *mut f64,
            a: *mut f64,
            mapId: *mut ::std::os::raw::c_int,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetPositionOfMap: Result<
        unsafe extern "C" fn(x: *mut f64, y: *mut f64, mapId: *mut ::std::os::raw::c_int) -> bool,
        ::libloading::Error,
    >,
    pub GetDirection: Result<unsafe extern "C" fn(a: *mut f64) -> bool, ::libloading::Error>,
    pub GetRotation: Result<unsafe extern "C" fn(a: *mut f64) -> bool, ::libloading::Error>,
    pub GetStar: Result<
        unsafe extern "C" fn(x: *mut f64, y: *mut f64, isEnd: *mut bool) -> bool,
        ::libloading::Error,
    >,
    pub GetStarJson: Result<
        unsafe extern "C" fn(jsonBuff: *mut ::std::os::raw::c_char) -> bool,
        ::libloading::Error,
    >,
    pub GetUID:
        Result<unsafe extern "C" fn(uid: *mut ::std::os::raw::c_int) -> bool, ::libloading::Error>,
    pub GetAllInfo: Result<
        unsafe extern "C" fn(
            x: *mut f64,
            y: *mut f64,
            mapId: *mut ::std::os::raw::c_int,
            a: *mut f64,
            r: *mut f64,
            uid: *mut ::std::os::raw::c_int,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetInfoLoadPicture: Result<
        unsafe extern "C" fn(
            path: *mut ::std::os::raw::c_char,
            uid: *mut ::std::os::raw::c_int,
            x: *mut f64,
            y: *mut f64,
            a: *mut f64,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetInfoLoadVideo: Result<
        unsafe extern "C" fn(
            path: *mut ::std::os::raw::c_char,
            pathOutFile: *mut ::std::os::raw::c_char,
        ) -> bool,
        ::libloading::Error,
    >,
    pub DebugCapture: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub DebugCapturePath: Result<
        unsafe extern "C" fn(
            path_buff: *const ::std::os::raw::c_char,
            buff_size: ::std::os::raw::c_int,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetLastErr: Result<unsafe extern "C" fn() -> ::std::os::raw::c_int, ::libloading::Error>,
    pub GetLastErrMsg: Result<
        unsafe extern "C" fn(
            msg_buff: *mut ::std::os::raw::c_char,
            buff_size: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
        ::libloading::Error,
    >,
    pub GetLastErrJson: Result<
        unsafe extern "C" fn(
            json_buff: *mut ::std::os::raw::c_char,
            buff_size: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
        ::libloading::Error,
    >,
    pub SetDisableFileLog: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub SetEnableFileLog: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
    pub GetCompileVersion: Result<
        unsafe extern "C" fn(
            version_buff: *mut ::std::os::raw::c_char,
            buff_size: ::std::os::raw::c_int,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetCompileTime: Result<
        unsafe extern "C" fn(
            time_buff: *mut ::std::os::raw::c_char,
            buff_size: ::std::os::raw::c_int,
        ) -> bool,
        ::libloading::Error,
    >,
    pub GetMapIsEmbedded: Result<unsafe extern "C" fn() -> bool, ::libloading::Error>,
}
impl cvAutoTrack {
    pub unsafe fn new<P>(path: P) -> Result<Self, ::libloading::Error>
    where
        P: AsRef<::std::ffi::OsStr>,
    {
        let library = ::libloading::Library::new(path)?;
        Self::from_library(library)
    }
    pub unsafe fn from_library<L>(library: L) -> Result<Self, ::libloading::Error>
    where
        L: Into<::libloading::Library>,
    {
        let __library = library.into();
        let verison = __library.get(b"verison\0").map(|sym| *sym);
        let init = __library.get(b"init\0").map(|sym| *sym);
        let uninit = __library.get(b"uninit\0").map(|sym| *sym);
        let startServe = __library.get(b"startServe\0").map(|sym| *sym);
        let stopServe = __library.get(b"stopServe\0").map(|sym| *sym);
        let SetUseBitbltCaptureMode = __library.get(b"SetUseBitbltCaptureMode\0").map(|sym| *sym);
        let SetUseDx11CaptureMode = __library.get(b"SetUseDx11CaptureMode\0").map(|sym| *sym);
        let SetHandle = __library.get(b"SetHandle\0").map(|sym| *sym);
        let SetWorldCenter = __library.get(b"SetWorldCenter\0").map(|sym| *sym);
        let SetWorldScale = __library.get(b"SetWorldScale\0").map(|sym| *sym);
        let ImportMapBlock = __library.get(b"ImportMapBlock\0").map(|sym| *sym);
        let ImportMapBlockCenter = __library.get(b"ImportMapBlockCenter\0").map(|sym| *sym);
        let ImportMapBlockCenterScale = __library
            .get(b"ImportMapBlockCenterScale\0")
            .map(|sym| *sym);
        let GetTransformOfMap = __library.get(b"GetTransformOfMap\0").map(|sym| *sym);
        let GetPositionOfMap = __library.get(b"GetPositionOfMap\0").map(|sym| *sym);
        let GetDirection = __library.get(b"GetDirection\0").map(|sym| *sym);
        let GetRotation = __library.get(b"GetRotation\0").map(|sym| *sym);
        let GetStar = __library.get(b"GetStar\0").map(|sym| *sym);
        let GetStarJson = __library.get(b"GetStarJson\0").map(|sym| *sym);
        let GetUID = __library.get(b"GetUID\0").map(|sym| *sym);
        let GetAllInfo = __library.get(b"GetAllInfo\0").map(|sym| *sym);
        let GetInfoLoadPicture = __library.get(b"GetInfoLoadPicture\0").map(|sym| *sym);
        let GetInfoLoadVideo = __library.get(b"GetInfoLoadVideo\0").map(|sym| *sym);
        let DebugCapture = __library.get(b"DebugCapture\0").map(|sym| *sym);
        let DebugCapturePath = __library.get(b"DebugCapturePath\0").map(|sym| *sym);
        let GetLastErr = __library.get(b"GetLastErr\0").map(|sym| *sym);
        let GetLastErrMsg = __library.get(b"GetLastErrMsg\0").map(|sym| *sym);
        let GetLastErrJson = __library.get(b"GetLastErrJson\0").map(|sym| *sym);
        let SetDisableFileLog = __library.get(b"SetDisableFileLog\0").map(|sym| *sym);
        let SetEnableFileLog = __library.get(b"SetEnableFileLog\0").map(|sym| *sym);
        let GetCompileVersion = __library.get(b"GetCompileVersion\0").map(|sym| *sym);
        let GetCompileTime = __library.get(b"GetCompileTime\0").map(|sym| *sym);
        let GetMapIsEmbedded = __library.get(b"GetMapIsEmbedded\0").map(|sym| *sym);
        Ok(cvAutoTrack {
            __library,
            verison,
            init,
            uninit,
            startServe,
            stopServe,
            SetUseBitbltCaptureMode,
            SetUseDx11CaptureMode,
            SetHandle,
            SetWorldCenter,
            SetWorldScale,
            ImportMapBlock,
            ImportMapBlockCenter,
            ImportMapBlockCenterScale,
            GetTransformOfMap,
            GetPositionOfMap,
            GetDirection,
            GetRotation,
            GetStar,
            GetStarJson,
            GetUID,
            GetAllInfo,
            GetInfoLoadPicture,
            GetInfoLoadVideo,
            DebugCapture,
            DebugCapturePath,
            GetLastErr,
            GetLastErrMsg,
            GetLastErrJson,
            SetDisableFileLog,
            SetEnableFileLog,
            GetCompileVersion,
            GetCompileTime,
            GetMapIsEmbedded,
        })
    }
    pub unsafe fn verison(&self, versionBuff: *mut ::std::os::raw::c_char) -> bool {
        (self
            .verison
            .as_ref()
            .expect("Expected function, got error."))(versionBuff)
    }
    pub unsafe fn init(&self) -> bool {
        (self.init.as_ref().expect("Expected function, got error."))()
    }
    pub unsafe fn uninit(&self) -> bool {
        (self.uninit.as_ref().expect("Expected function, got error."))()
    }
    pub unsafe fn startServe(&self) -> bool {
        (self
            .startServe
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn stopServe(&self) -> bool {
        (self
            .stopServe
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn SetUseBitbltCaptureMode(&self) -> bool {
        (self
            .SetUseBitbltCaptureMode
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn SetUseDx11CaptureMode(&self) -> bool {
        (self
            .SetUseDx11CaptureMode
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn SetHandle(&self, handle: ::std::os::raw::c_longlong) -> bool {
        (self
            .SetHandle
            .as_ref()
            .expect("Expected function, got error."))(handle)
    }
    pub unsafe fn SetWorldCenter(&self, x: f64, y: f64) -> bool {
        (self
            .SetWorldCenter
            .as_ref()
            .expect("Expected function, got error."))(x, y)
    }
    pub unsafe fn SetWorldScale(&self, scale: f64) -> bool {
        (self
            .SetWorldScale
            .as_ref()
            .expect("Expected function, got error."))(scale)
    }
    pub unsafe fn ImportMapBlock(
        &self,
        id_x: ::std::os::raw::c_int,
        id_y: ::std::os::raw::c_int,
        image_data: *const ::std::os::raw::c_char,
        image_data_size: ::std::os::raw::c_int,
        image_width: ::std::os::raw::c_int,
        image_height: ::std::os::raw::c_int,
    ) -> bool {
        (self
            .ImportMapBlock
            .as_ref()
            .expect("Expected function, got error."))(
            id_x,
            id_y,
            image_data,
            image_data_size,
            image_width,
            image_height,
        )
    }
    pub unsafe fn ImportMapBlockCenter(
        &self,
        x: ::std::os::raw::c_int,
        y: ::std::os::raw::c_int,
    ) -> bool {
        (self
            .ImportMapBlockCenter
            .as_ref()
            .expect("Expected function, got error."))(x, y)
    }
    pub unsafe fn ImportMapBlockCenterScale(
        &self,
        x: ::std::os::raw::c_int,
        y: ::std::os::raw::c_int,
        scale: f64,
    ) -> bool {
        (self
            .ImportMapBlockCenterScale
            .as_ref()
            .expect("Expected function, got error."))(x, y, scale)
    }
    pub unsafe fn GetTransformOfMap(
        &self,
        x: *mut f64,
        y: *mut f64,
        a: *mut f64,
        mapId: *mut ::std::os::raw::c_int,
    ) -> bool {
        (self
            .GetTransformOfMap
            .as_ref()
            .expect("Expected function, got error."))(x, y, a, mapId)
    }
    pub unsafe fn GetPositionOfMap(
        &self,
        x: *mut f64,
        y: *mut f64,
        mapId: *mut ::std::os::raw::c_int,
    ) -> bool {
        (self
            .GetPositionOfMap
            .as_ref()
            .expect("Expected function, got error."))(x, y, mapId)
    }
    pub unsafe fn GetDirection(&self, a: *mut f64) -> bool {
        (self
            .GetDirection
            .as_ref()
            .expect("Expected function, got error."))(a)
    }
    pub unsafe fn GetRotation(&self, a: *mut f64) -> bool {
        (self
            .GetRotation
            .as_ref()
            .expect("Expected function, got error."))(a)
    }
    pub unsafe fn GetStar(&self, x: *mut f64, y: *mut f64, isEnd: *mut bool) -> bool {
        (self
            .GetStar
            .as_ref()
            .expect("Expected function, got error."))(x, y, isEnd)
    }
    pub unsafe fn GetStarJson(&self, jsonBuff: *mut ::std::os::raw::c_char) -> bool {
        (self
            .GetStarJson
            .as_ref()
            .expect("Expected function, got error."))(jsonBuff)
    }
    pub unsafe fn GetUID(&self, uid: *mut ::std::os::raw::c_int) -> bool {
        (self.GetUID.as_ref().expect("Expected function, got error."))(uid)
    }
    pub unsafe fn GetAllInfo(
        &self,
        x: *mut f64,
        y: *mut f64,
        mapId: *mut ::std::os::raw::c_int,
        a: *mut f64,
        r: *mut f64,
        uid: *mut ::std::os::raw::c_int,
    ) -> bool {
        (self
            .GetAllInfo
            .as_ref()
            .expect("Expected function, got error."))(x, y, mapId, a, r, uid)
    }
    pub unsafe fn GetInfoLoadPicture(
        &self,
        path: *mut ::std::os::raw::c_char,
        uid: *mut ::std::os::raw::c_int,
        x: *mut f64,
        y: *mut f64,
        a: *mut f64,
    ) -> bool {
        (self
            .GetInfoLoadPicture
            .as_ref()
            .expect("Expected function, got error."))(path, uid, x, y, a)
    }
    pub unsafe fn GetInfoLoadVideo(
        &self,
        path: *mut ::std::os::raw::c_char,
        pathOutFile: *mut ::std::os::raw::c_char,
    ) -> bool {
        (self
            .GetInfoLoadVideo
            .as_ref()
            .expect("Expected function, got error."))(path, pathOutFile)
    }
    pub unsafe fn DebugCapture(&self) -> bool {
        (self
            .DebugCapture
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn DebugCapturePath(
        &self,
        path_buff: *const ::std::os::raw::c_char,
        buff_size: ::std::os::raw::c_int,
    ) -> bool {
        (self
            .DebugCapturePath
            .as_ref()
            .expect("Expected function, got error."))(path_buff, buff_size)
    }
    pub unsafe fn GetLastErr(&self) -> ::std::os::raw::c_int {
        (self
            .GetLastErr
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn GetLastErrMsg(
        &self,
        msg_buff: *mut ::std::os::raw::c_char,
        buff_size: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        (self
            .GetLastErrMsg
            .as_ref()
            .expect("Expected function, got error."))(msg_buff, buff_size)
    }
    pub unsafe fn GetLastErrJson(
        &self,
        json_buff: *mut ::std::os::raw::c_char,
        buff_size: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        (self
            .GetLastErrJson
            .as_ref()
            .expect("Expected function, got error."))(json_buff, buff_size)
    }
    pub unsafe fn SetDisableFileLog(&self) -> bool {
        (self
            .SetDisableFileLog
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn SetEnableFileLog(&self) -> bool {
        (self
            .SetEnableFileLog
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub unsafe fn GetCompileVersion(
        &self,
        version_buff: *mut ::std::os::raw::c_char,
        buff_size: ::std::os::raw::c_int,
    ) -> bool {
        (self
            .GetCompileVersion
            .as_ref()
            .expect("Expected function, got error."))(version_buff, buff_size)
    }
    pub unsafe fn GetCompileTime(
        &self,
        time_buff: *mut ::std::os::raw::c_char,
        buff_size: ::std::os::raw::c_int,
    ) -> bool {
        (self
            .GetCompileTime
            .as_ref()
            .expect("Expected function, got error."))(time_buff, buff_size)
    }
    #[doc = " <summary>\n whether map image resources are embedded in dll\n </summary>\n <returns>true or false</returns>"]
    pub unsafe fn GetMapIsEmbedded(&self) -> bool {
        (self
            .GetMapIsEmbedded
            .as_ref()
            .expect("Expected function, got error."))()
    }
    pub fn close(self) {
        drop(self.__library);
    }
}