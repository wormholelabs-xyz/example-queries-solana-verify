use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub struct QueryRequest {
    pub version: u8,
    pub nonce: u32,
    pub requests: Vec<PerChainQueryRequest>,
}

impl QueryRequest {
    pub const REQUEST_VERSION: u8 = 1;

    pub fn deserialize(data: &[u8]) -> std::result::Result<QueryRequest, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<QueryRequest, std::io::Error> {
        let version = rdr.read_u8()?;
        if version != Self::REQUEST_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "VersionMismatch",
            ));
        }

        let nonce = rdr.read_u32::<BigEndian>()?;

        let num_per_chain_queries = rdr.read_u8()?;

        // A valid query request has at least one per chain query
        if num_per_chain_queries == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ZeroQueries",
            ));
        }

        let mut requests: Vec<PerChainQueryRequest> =
            Vec::with_capacity(num_per_chain_queries.into());
        for _idx in 0..num_per_chain_queries {
            requests.push(PerChainQueryRequest::deserialize_from_reader(rdr)?)
        }

        Ok(QueryRequest {
            version,
            nonce,
            requests,
        })
    }
}

pub struct PerChainQueryRequest {
    pub chain_id: u16,
    pub query: ChainSpecificQuery,
}

impl PerChainQueryRequest {
    pub fn deserialize(data: &[u8]) -> std::result::Result<PerChainQueryRequest, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<PerChainQueryRequest, std::io::Error> {
        let chain_id = rdr.read_u16::<BigEndian>()?;
        let query_type = rdr.read_u8()?;
        rdr.read_u32::<BigEndian>()?; // skip the query length

        let query: ChainSpecificQuery;
        if query_type == 1 {
            query = ChainSpecificQuery::EthCallQueryRequest(
                EthCallQueryRequest::deserialize_from_reader(rdr)?,
            );
        } else if query_type == 2 {
            query = ChainSpecificQuery::EthCallByTimestampQueryRequest(
                EthCallByTimestampQueryRequest::deserialize_from_reader(rdr)?,
            );
        } else if query_type == 3 {
            query = ChainSpecificQuery::EthCallWithFinalityQueryRequest(
                EthCallWithFinalityQueryRequest::deserialize_from_reader(rdr)?,
            );
        } else if query_type == 4 {
            query = ChainSpecificQuery::SolanaAccountQueryRequest(
                SolanaAccountQueryRequest::deserialize_from_reader(rdr)?,
            );
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "UnsupportedQueryType",
            ));
        }

        Ok(PerChainQueryRequest { chain_id, query })
    }
}

pub enum ChainSpecificQuery {
    EthCallQueryRequest(EthCallQueryRequest),
    EthCallByTimestampQueryRequest(EthCallByTimestampQueryRequest),
    EthCallWithFinalityQueryRequest(EthCallWithFinalityQueryRequest),
    SolanaAccountQueryRequest(SolanaAccountQueryRequest),
}

pub struct EthCallQueryRequest {
    pub block_tag: String,
    pub call_data: Vec<EthCallData>,
}

pub struct EthCallData {
    pub to: [u8; 20],
    pub data: Vec<u8>,
}

impl EthCallQueryRequest {
    pub fn deserialize(data: &[u8]) -> std::result::Result<EthCallQueryRequest, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<EthCallQueryRequest, std::io::Error> {
        let block_tag_len = rdr.read_u32::<BigEndian>()?;
        let mut buf = vec![0u8; block_tag_len.try_into().unwrap()];
        rdr.read_exact(&mut buf)?;
        let block_tag = String::from_utf8(buf.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "InvalidBlockTag"))?;
        let call_data_len = rdr.read_u8()?;
        let mut call_data = Vec::with_capacity(call_data_len.into());
        for _ in 0..call_data_len {
            let mut to = [0u8; 20];
            rdr.read_exact(&mut to)?;
            let data_len = rdr.read_u32::<BigEndian>()?;
            let mut data = vec![0u8; data_len.try_into().unwrap()];
            rdr.read_exact(&mut data)?;
            call_data.push(EthCallData { to, data })
        }
        Ok(EthCallQueryRequest {
            block_tag,
            call_data,
        })
    }
}

pub struct EthCallByTimestampQueryRequest {
    pub target_timestamp: u64,
    pub target_block_hint: String,
    pub following_block_hint: String,
    pub call_data: Vec<EthCallData>,
}

impl EthCallByTimestampQueryRequest {
    pub fn deserialize(
        data: &[u8],
    ) -> std::result::Result<EthCallByTimestampQueryRequest, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<EthCallByTimestampQueryRequest, std::io::Error> {
        let target_timestamp = rdr.read_u64::<BigEndian>()?;
        let target_block_hint_len = rdr.read_u32::<BigEndian>()?;
        let mut buf = vec![0u8; target_block_hint_len.try_into().unwrap()];
        rdr.read_exact(&mut buf)?;
        let target_block_hint = String::from_utf8(buf.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "InvalidBlockTag"))?;
        let following_block_hint_len = rdr.read_u32::<BigEndian>()?;
        let mut buf = vec![0u8; following_block_hint_len.try_into().unwrap()];
        rdr.read_exact(&mut buf)?;
        let following_block_hint = String::from_utf8(buf.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "InvalidBlockTag"))?;
        let call_data_len = rdr.read_u8()?;
        let mut call_data = Vec::with_capacity(call_data_len.into());
        for _ in 0..call_data_len {
            let mut to = [0u8; 20];
            rdr.read_exact(&mut to)?;
            let data_len = rdr.read_u32::<BigEndian>()?;
            let mut data = vec![0u8; data_len.try_into().unwrap()];
            rdr.read_exact(&mut data)?;
            call_data.push(EthCallData { to, data })
        }
        Ok(EthCallByTimestampQueryRequest {
            target_timestamp,
            target_block_hint,
            following_block_hint,
            call_data,
        })
    }
}

pub struct EthCallWithFinalityQueryRequest {
    pub block_tag: String,
    pub finality: String,
    pub call_data: Vec<EthCallData>,
}

impl EthCallWithFinalityQueryRequest {
    pub fn deserialize(
        data: &[u8],
    ) -> std::result::Result<EthCallWithFinalityQueryRequest, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<EthCallWithFinalityQueryRequest, std::io::Error> {
        let block_tag_len = rdr.read_u32::<BigEndian>()?;
        let mut buf = vec![0u8; block_tag_len.try_into().unwrap()];
        rdr.read_exact(&mut buf)?;
        let block_tag = String::from_utf8(buf.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "InvalidBlockTag"))?;
        let finality_len = rdr.read_u32::<BigEndian>()?;
        let mut buf = vec![0u8; finality_len.try_into().unwrap()];
        rdr.read_exact(&mut buf)?;
        let finality = String::from_utf8(buf.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "InvalidFinality"))?;
        let call_data_len = rdr.read_u8()?;
        let mut call_data = Vec::with_capacity(call_data_len.into());
        for _ in 0..call_data_len {
            let mut to = [0u8; 20];
            rdr.read_exact(&mut to)?;
            let data_len = rdr.read_u32::<BigEndian>()?;
            let mut data = vec![0u8; data_len.try_into().unwrap()];
            rdr.read_exact(&mut data)?;
            call_data.push(EthCallData { to, data })
        }
        Ok(EthCallWithFinalityQueryRequest {
            block_tag,
            finality,
            call_data,
        })
    }
}

pub struct SolanaAccountQueryRequest {
    pub commitment: String,
    pub min_context_slot: u64,
    pub data_slice_offset: u64,
    pub data_slice_length: u64,
    pub accounts: Vec<[u8; 32]>,
}

impl SolanaAccountQueryRequest {
    pub fn deserialize(
        data: &[u8],
    ) -> std::result::Result<SolanaAccountQueryRequest, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<SolanaAccountQueryRequest, std::io::Error> {
        let commitment_len = rdr.read_u32::<BigEndian>()?;
        let mut buf = vec![0u8; commitment_len.try_into().unwrap()];
        rdr.read_exact(&mut buf)?;
        let commitment = String::from_utf8(buf.clone())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "InvalidBlockTag"))?;
        let min_context_slot = rdr.read_u64::<BigEndian>()?;
        let data_slice_offset = rdr.read_u64::<BigEndian>()?;
        let data_slice_length = rdr.read_u64::<BigEndian>()?;
        let accounts_len = rdr.read_u8()?;
        let mut accounts = Vec::with_capacity(accounts_len.into());
        for _ in 0..accounts_len {
            let mut account = [0u8; 32];
            rdr.read_exact(&mut account)?;
            accounts.push(account)
        }
        Ok(SolanaAccountQueryRequest {
            commitment,
            min_context_slot,
            data_slice_offset,
            data_slice_length,
            accounts,
        })
    }
}
