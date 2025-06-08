#![allow(dead_code, unused_imports)]

pub(crate) use anyhow::Result;
pub(crate) use sqlx::Database as SqlxDatabase;
pub(crate) use sqlx::Decode as SqlxDecode;
pub(crate) use sqlx::Encode as SqlxEncode;
pub(crate) use sqlx::Type as SqlxType;
pub(crate) use sqlx::encode::IsNull;
pub(crate) use sqlx::error::BoxDynError;

pub(crate) type BoxDynResult<T> = Result<T, BoxDynError>;

pub(crate) trait Type {
    type Proxy;

    // Required.
    fn into_proxy(self) -> Result<Self::Proxy>;
    fn to_proxy(&self) -> Result<Self::Proxy>;
}

pub(crate) trait SizedType: Type + Sized {
    fn from_proxy(proxy: Self::Proxy) -> Result<Self>;
}

#[macro_export]
macro_rules! sqlx_type_impl {
    ($type:ty) => {
        impl<DB> $crate::proxied::SqlxType<DB> for $type
        where
            DB: $crate::proxied::SqlxDatabase,
            Self: $crate::proxied::Type,
            <Self as $crate::proxied::Type>::Proxy: $crate::proxied::SqlxType<DB>,
        {
            fn type_info() -> <DB as $crate::proxied::SqlxDatabase>::TypeInfo {
                type Proxy = <$type as $crate::proxied::Type>::Proxy;
                <Proxy as $crate::proxied::SqlxType<DB>>::type_info()
            }
        }
    };
}

#[macro_export]
macro_rules! sqlx_decode_impl {
    ($type:ty) => {
        impl<'r, DB> $crate::proxied::SqlxDecode<'r, DB> for $type
        where
            DB: $crate::proxied::SqlxDatabase,
            Self: $crate::proxied::SizedType,
            <Self as $crate::proxied::Type>::Proxy: $crate::proxied::SqlxDecode<'r, DB>,
        {
            fn decode(
                value: <DB as $crate::proxied::SqlxDatabase>::ValueRef<'r>,
            ) -> BoxDynResult<Self> {
                type Proxy = <$type as $crate::proxied::Type>::Proxy;
                let proxy = <Proxy as $crate::proxied::SqlxDecode<'r, DB>>::decode(value)?;
                Ok(<Self as $crate::proxied::SizedType>::from_proxy(proxy)?)
            }
        }
    };
}

#[macro_export]
macro_rules! sqlx_encode_impl {
    ($type:ty) => {
        impl<'q, DB> $crate::proxied::SqlxEncode<'q, DB> for $type
        where
            DB: $crate::proxied::SqlxDatabase,
            Self: $crate::proxied::Type,
            <Self as $crate::proxied::Type>::Proxy: $crate::proxied::SqlxEncode<'q, DB>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut <DB as $crate::proxied::SqlxDatabase>::ArgumentBuffer<'q>,
            ) -> $crate::proxied::BoxDynResult<$crate::proxied::IsNull> {
                let proxy = self.to_proxy()?;
                proxy.encode_by_ref(buf)
            }
            fn encode(
                self,
                buf: &mut <DB as $crate::proxied::SqlxDatabase>::ArgumentBuffer<'q>,
            ) -> $crate::proxied::BoxDynResult<$crate::proxied::IsNull>
            where
                Self: Sized,
            {
                let proxy = self.into_proxy()?;
                proxy.encode(buf)
            }
        }
    };
}

#[macro_export]
macro_rules! sqlx_impl {
    ($type:ty) => {
        $crate::sqlx_type_impl!($type);
        $crate::sqlx_decode_impl!($type);
        $crate::sqlx_encode_impl!($type);
    };
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use super::*;

    struct TestType;

    impl Type for TestType {
        type Proxy = Vec<u8>;

        fn into_proxy(self) -> Result<Self::Proxy> {
            Ok(Vec::new())
        }
        fn to_proxy(&self) -> Result<Self::Proxy> {
            Ok(Vec::new())
        }
    }

    impl SizedType for TestType {
        fn from_proxy(_proxy: Self::Proxy) -> Result<Self> {
            Ok(Self)
        }
    }

    sqlx_impl!(TestType);

    #[test]
    fn test_sqlite() {
        fn assert_type<T: SqlxType<Sqlite>>() {}
        fn assert_decode<'r, T: SqlxDecode<'r, Sqlite>>() {}
        fn assert_encode<'q, T: SqlxEncode<'q, Sqlite>>() {}

        assert_type::<TestType>();
        assert_decode::<TestType>();
        assert_encode::<TestType>();
    }
}
