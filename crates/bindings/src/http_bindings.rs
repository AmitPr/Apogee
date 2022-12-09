wit_bindgen_guest_rust::generate!({path:"./wit/http_service.wit", macro_export});
#[cfg(feature="import")]
pub(crate) mod imports {
    wasmtime::component::bindgen!({
        path: "./wit/http_service.wit",
    });

    pub use http_import::add_to_linker;

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

    impl Response {
        pub fn into_hyper_response<T>(self) -> hyper::Response<T>
        where
            T: Default + From<Vec<u8>>,
        {
            let mut res = hyper::Response::builder()
                .status(self.status)
                .version(self.version.into());
            for header in self.headers {
                res = res.header(header.key, header.value);
            }
            res.body(T::from(self.body)).unwrap()
        }

        pub fn try_into_hyper_response<T>(self) -> Result<hyper::Response<T>, T::Error>
        where
            T: Default + TryFrom<Vec<u8>>,
        {
            let mut res = hyper::Response::builder()
                .status(self.status)
                .version(self.version.into());
            for header in self.headers {
                res = res.header(header.key, header.value);
            }
            Ok(res.body(T::try_from(self.body)?).unwrap())
        }


        pub fn response_builder(self) -> hyper::http::response::Builder
        {
            let mut res = hyper::Response::builder()
                .status(self.status)
                .version(self.version.into());
            for header in self.headers {
                res = res.header(header.key, header.value);
            }
            res
        }
    }

    impl<T> TryInto<hyper::Response<T>> for Response
    where
        T: Default + From<Vec<u8>>,
    {
        type Error = String;
        fn try_into(self) -> Result<hyper::Response<T>, Self::Error> {
            let mut res = hyper::Response::builder()
                .status(self.status)
                .version(self.version.into());
            for header in self.headers {
                res = res.header(header.key, header.value);
            }
            Ok(res.body(T::from(self.body)).unwrap())
        }
    }
}
