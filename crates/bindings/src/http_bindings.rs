wit_bindgen_guest_rust::generate!({path:"./wit/http_service.wit", macro_export});
#[cfg(feature="import")]
pub(crate) mod imports {
    wit_bindgen_host_wasmtime_rust::generate!({
        path: "./wit/http_service.wit",
    });

    impl TryFrom<hyper::Method> for Method {
        type Error = String;
        fn try_from(method: hyper::Method) -> Result<Self, Self::Error> {
            match method {
                hyper::Method::OPTIONS => Ok(Method::Options),
                hyper::Method::GET => Ok(Method::Get),
                hyper::Method::POST => Ok(Method::Post),
                hyper::Method::PUT => Ok(Method::Put),
                hyper::Method::DELETE => Ok(Method::Delete),
                hyper::Method::HEAD => Ok(Method::Head),
                hyper::Method::TRACE => Ok(Method::Trace),
                hyper::Method::CONNECT => Ok(Method::Connect),
                hyper::Method::PATCH => Ok(Method::Patch),
                _ => Err("Unknown method".to_string()),
            }
        }
    }

    impl TryFrom<hyper::Version> for Version {
        type Error = String;
        fn try_from(version: hyper::Version) -> Result<Self, Self::Error> {
            match version {
                hyper::Version::HTTP_09 => Ok(Version::HttpV09),
                hyper::Version::HTTP_10 => Ok(Version::HttpV10),
                hyper::Version::HTTP_11 => Ok(Version::HttpV11),
                hyper::Version::HTTP_2 => Ok(Version::HttpV2),
                hyper::Version::HTTP_3 => Ok(Version::HttpV3),
                _ => Err("Unknown version".to_string()),
            }
        }
    }

    impl From<Version> for hyper::Version {
        fn from(val: Version) -> Self {
            match val {
                Version::HttpV09 => hyper::Version::HTTP_09,
                Version::HttpV10 => hyper::Version::HTTP_10,
                Version::HttpV11 => hyper::Version::HTTP_11,
                Version::HttpV2 => hyper::Version::HTTP_2,
                Version::HttpV3 => hyper::Version::HTTP_3,
            }
        }
    }

    // impl<'a, T: 'a> TryInto<Request<'a>> for &'a hyper::Request<T> {
    //     type Error = String;

    //     fn try_into(self) -> Result<Request<'a>, Self::Error> {
    //         let method = Method::try_from(self.method().clone())?;
    //         let version = Version::try_from(self.version())?;
    //         let uri: &'a str = self.uri().path();
    //         let headers: Vec<HeaderParam<'a>> = self
    //             .headers()
    //             .iter()
    //             .map(|(key, value)| {
    //                 let header: HeaderParam<'a> = HeaderParam {
    //                     key: key.as_str().as_bytes(),
    //                     value: value.as_bytes(),
    //                 };
    //                 header
    //             })
    //             .collect();
    //         let headers: &'a [HeaderParam<'a>] = headers.as_slice();
    //         Ok(Request {
    //             method,
    //             uri,
    //             version,
    //             headers,
    //         })
    //     }
    // }

    impl<T> TryInto<hyper::Response<T>> for Response
    where
        T: Default,
    {
        type Error = String;
        fn try_into(self) -> Result<hyper::Response<T>, Self::Error> {
            let mut res = hyper::Response::builder()
                .status(self.status)
                .version(self.version.into());
            for header in self.headers {
                res = res.header(header.key, header.value);
            }
            Ok(res.body(T::default()).unwrap())
        }
    }
}
