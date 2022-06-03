use crate::Error;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Device {
    path: String,
    attrs: Option<HashSet<String>>,
}

impl Device {
    #[inline]
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            attrs: None,
        }
    }
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[inline]
    /// # Safety
    ///
    /// Consider with libowcapi safety
    pub unsafe fn load(&mut self) -> Result<(), Error> {
        let data = crate::get(&self.path)?;
        let a: HashSet<String> = data
            .split(',')
            .filter(|v| !v.ends_with('/'))
            .map(ToOwned::to_owned)
            .collect();
        self.attrs.replace(a);
        Ok(())
    }
    #[inline]
    pub fn attrs(&self) -> Vec<&str> {
        if let Some(ref a) = self.attrs {
            a.iter().map(String::as_str).collect()
        } else {
            Vec::new()
        }
    }
    #[inline]
    pub fn has(&self, attr: &str) -> bool {
        if let Some(ref a) = self.attrs {
            a.contains(attr)
        } else {
            false
        }
    }
    #[inline]
    /// # Safety
    ///
    /// Consider with libowcapi safety
    pub unsafe fn get(&self, attr: &str) -> Result<String, Error> {
        crate::get(&format!("{}/{}", self.path, attr))
    }
    #[inline]
    /// # Safety
    ///
    /// Consider with libowcapi safety
    pub unsafe fn set(&self, attr: &str, value: &str) -> Result<(), Error> {
        crate::set(&format!("{}/{}", self.path, attr), value)
    }
    #[inline]
    /// # Safety
    ///
    /// Consider with libowcapi safety
    pub unsafe fn info(&self) -> Result<DeviceInfo, Error> {
        Ok(DeviceInfo {
            w1_type: self.get("type")?,
            path: &self.path,
            family: self.get("family")?.parse().ok(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo<'a> {
    w1_type: String,
    path: &'a str,
    family: Option<u32>,
}

impl<'a> DeviceInfo<'a> {
    #[inline]
    pub fn w1_type(&self) -> &str {
        &self.w1_type
    }
    #[inline]
    pub fn path(&self) -> &str {
        self.path
    }
    #[inline]
    pub fn family(&self) -> Option<u32> {
        self.family
    }
}
