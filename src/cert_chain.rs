//! Bindings to winapi's certificate-chain related APIs.

use std::mem;
use std::slice;
use crypt32;
use winapi;

use cert_context::CertContext;
use Inner;

pub struct CertChainContext(pub winapi::PCERT_CHAIN_CONTEXT);

impl Clone for CertChainContext {
    fn clone(&self) -> Self {
        let rced = unsafe {
            crypt32::CertDuplicateCertificateChain(self.0) as *mut _
        };
        CertChainContext(rced)
    }
}

impl Drop for CertChainContext {
    fn drop(&mut self) {
        unsafe {
            crypt32::CertFreeCertificateChain(self.0);
        }
    }
}

impl CertChainContext {
    /// Get the final (for a successful verification this means successful) certificate chain
    ///
    /// https://msdn.microsoft.com/de-de/library/windows/desktop/aa377182(v=vs.85).aspx
    /// rgpChain[cChain - 1] is the final chain
    pub fn final_chain(&self) -> Option<CertChain> {
        let cloned = self.clone();
        let chains = unsafe {
            let cert_chain = *cloned.0;
            slice::from_raw_parts(
                cert_chain.rgpChain as *mut winapi::PCERT_SIMPLE_CHAIN,
                cert_chain.cChain as usize)
        };
        chains.last().map(|chain| CertChain(*chain, cloned))
    }
}

pub struct CertChain(winapi::PCERT_SIMPLE_CHAIN, CertChainContext);

impl CertChain {
    /// Returns the number of certificates in the chain
    pub fn len(&self) -> usize {
        unsafe {
            (*self.0).cElement as usize
        }
    }

    /// Get the n-th certificate from the current chain
    pub fn get(&self, idx: usize) -> Option<CertContext> {
        let elements = unsafe {
            let cert_chain = *self.0;
            slice::from_raw_parts(
                cert_chain.rgpElement as *mut &mut winapi::CERT_CHAIN_ELEMENT,
                cert_chain.cElement as usize)
        };
        elements.get(idx).map(|el| {
            let cert = unsafe {
                CertContext::from_inner(el.pCertContext)
            };
            let rc_cert = cert.clone();
            mem::forget(cert);
            rc_cert
        })
    }

    /// Return an iterator over all certificates in this chain
    pub fn certificates(&self) -> Certificates {
        Certificates {
            chain: self,
            idx: 0,
        }
    }
}

pub struct Certificates<'a> {
    chain: &'a CertChain,
    idx: usize,
}

impl<'a> Iterator for Certificates<'a> {
    type Item = CertContext;

    fn next(&mut self) -> Option<CertContext> {
        let idx = self.idx;
        self.idx += 1;
        self.chain.get(idx)
    }
}
