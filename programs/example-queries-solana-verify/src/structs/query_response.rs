use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use super::QueryRequest;

pub struct QueryResponse {
    pub version: u8,
    pub request_chain_id: u16,
    pub request_id: Vec<u8>,
    pub request: QueryRequest,
    pub responses: Vec<PerChainQueryResponse>,
}

impl QueryResponse {
    pub const RESPONSE_VERSION: u8 = 1;

    pub fn deserialize(data: &[u8]) -> std::result::Result<QueryResponse, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<QueryResponse, std::io::Error> {
        let version = rdr.read_u8()?;
        if version != Self::RESPONSE_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InvalidResponseVersion",
            ));
        }

        // For off chain requests (chainID zero), the requestId is the 65 byte signature. For on chain requests, it is the 32 byte VAA hash.
        let request_chain_id = rdr.read_u16::<BigEndian>()?;
        let request_id_len = if request_chain_id == 0 { 65 } else { 32 };
        let mut request_id = vec![0u8; request_id_len];
        rdr.read_exact(&mut request_id)?;

        rdr.read_u32::<BigEndian>()?; // skip the request length

        let request = QueryRequest::deserialize_from_reader(rdr)?;

        let num_per_chain_responses = rdr.read_u8()?;

        let mut responses: Vec<PerChainQueryResponse> =
            Vec::with_capacity(num_per_chain_responses.into());
        for _idx in 0..num_per_chain_responses {
            responses.push(PerChainQueryResponse::deserialize_from_reader(rdr)?)
        }

        if rdr.position() != rdr.get_ref().len() as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InvalidPayloadLength",
            ));
        }

        Ok(QueryResponse {
            version,
            request_chain_id,
            request_id,
            request,
            responses,
        })
    }
}

pub struct PerChainQueryResponse {
    pub chain_id: u16,
    pub response: ChainSpecificResponse,
}

impl PerChainQueryResponse {
    pub fn deserialize(data: &[u8]) -> std::result::Result<PerChainQueryResponse, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<PerChainQueryResponse, std::io::Error> {
        let chain_id = rdr.read_u16::<BigEndian>()?;
        let query_type = rdr.read_u8()?;
        rdr.read_u32::<BigEndian>()?; // skip the response length

        let response: ChainSpecificResponse;
        if query_type == 1 {
            response = ChainSpecificResponse::EthCallQueryResponse(
                EthCallQueryResponse::deserialize_from_reader(rdr)?,
            );
        } else if query_type == 2 {
            response = ChainSpecificResponse::EthCallByTimestampQueryResponse(
                EthCallByTimestampQueryResponse::deserialize_from_reader(rdr)?,
            );
        } else if query_type == 3 {
            response = ChainSpecificResponse::EthCallWithFinalityQueryResponse(
                EthCallWithFinalityQueryResponse::deserialize_from_reader(rdr)?,
            );
        } else if query_type == 4 {
            response = ChainSpecificResponse::SolanaAccountQueryResponse(
                SolanaAccountQueryResponse::deserialize_from_reader(rdr)?,
            );
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "UnsupportedResponseType",
            ));
        }

        Ok(PerChainQueryResponse { chain_id, response })
    }
}

pub enum ChainSpecificResponse {
    EthCallQueryResponse(EthCallQueryResponse),
    EthCallByTimestampQueryResponse(EthCallByTimestampQueryResponse),
    EthCallWithFinalityQueryResponse(EthCallWithFinalityQueryResponse),
    SolanaAccountQueryResponse(SolanaAccountQueryResponse),
}

pub struct EthCallQueryResponse {
    pub block_number: u64,
    pub block_hash: [u8; 32],
    pub block_time: u64,
    pub results: Vec<Vec<u8>>,
}

impl EthCallQueryResponse {
    pub fn deserialize(data: &[u8]) -> std::result::Result<EthCallQueryResponse, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<EthCallQueryResponse, std::io::Error> {
        let block_number = rdr.read_u64::<BigEndian>()?;
        let mut block_hash = [0u8; 32];
        rdr.read_exact(&mut block_hash)?;
        let block_time = rdr.read_u64::<BigEndian>()?;
        let results_len = rdr.read_u8()?;
        let mut results = Vec::with_capacity(results_len.into());
        for _ in 0..results_len {
            let result_len = rdr.read_u32::<BigEndian>()?;
            let mut result = vec![0u8; result_len.try_into().unwrap()];
            rdr.read_exact(&mut result)?;
            results.push(result)
        }
        Ok(EthCallQueryResponse {
            block_number,
            block_hash,
            block_time,
            results,
        })
    }
}

pub struct EthCallByTimestampQueryResponse {
    pub target_block_number: u64,
    pub target_block_hash: [u8; 32],
    pub target_block_time: u64,
    pub following_block_number: u64,
    pub following_block_hash: [u8; 32],
    pub following_block_time: u64,
    pub results: Vec<Vec<u8>>,
}

impl EthCallByTimestampQueryResponse {
    pub fn deserialize(
        data: &[u8],
    ) -> std::result::Result<EthCallByTimestampQueryResponse, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<EthCallByTimestampQueryResponse, std::io::Error> {
        let target_block_number = rdr.read_u64::<BigEndian>()?;
        let mut target_block_hash = [0u8; 32];
        rdr.read_exact(&mut target_block_hash)?;
        let target_block_time = rdr.read_u64::<BigEndian>()?;
        let following_block_number = rdr.read_u64::<BigEndian>()?;
        let mut following_block_hash = [0u8; 32];
        rdr.read_exact(&mut following_block_hash)?;
        let following_block_time = rdr.read_u64::<BigEndian>()?;
        let results_len = rdr.read_u8()?;
        let mut results = Vec::with_capacity(results_len.into());
        for _ in 0..results_len {
            let result_len = rdr.read_u32::<BigEndian>()?;
            let mut result = vec![0u8; result_len.try_into().unwrap()];
            rdr.read_exact(&mut result)?;
            results.push(result)
        }
        Ok(EthCallByTimestampQueryResponse {
            target_block_number,
            target_block_hash,
            target_block_time,
            following_block_number,
            following_block_hash,
            following_block_time,
            results,
        })
    }
}

pub struct EthCallWithFinalityQueryResponse {
    pub block_number: u64,
    pub block_hash: [u8; 32],
    pub block_time: u64,
    pub results: Vec<Vec<u8>>,
}

impl EthCallWithFinalityQueryResponse {
    pub fn deserialize(
        data: &[u8],
    ) -> std::result::Result<EthCallWithFinalityQueryResponse, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<EthCallWithFinalityQueryResponse, std::io::Error> {
        let EthCallQueryResponse {
            block_number,
            block_hash,
            block_time,
            results,
        } = EthCallQueryResponse::deserialize_from_reader(rdr)?;
        Ok(EthCallWithFinalityQueryResponse {
            block_number,
            block_hash,
            block_time,
            results,
        })
    }
}

pub struct SolanaAccountQueryResponse {
    pub slot_number: u64,
    pub block_time: u64,
    pub block_hash: [u8; 32],
    pub results: Vec<SolanaAccountResult>,
}

pub struct SolanaAccountResult {
    pub lamports: u64,
    pub rent_epoch: u64,
    pub executable: bool,
    pub owner: [u8; 32],
    pub data: Vec<u8>,
}

impl SolanaAccountQueryResponse {
    pub fn deserialize(
        data: &[u8],
    ) -> std::result::Result<SolanaAccountQueryResponse, std::io::Error> {
        let mut rdr = Cursor::new(data);
        Self::deserialize_from_reader(&mut rdr)
    }

    pub fn deserialize_from_reader(
        rdr: &mut Cursor<&[u8]>,
    ) -> std::result::Result<SolanaAccountQueryResponse, std::io::Error> {
        let slot_number = rdr.read_u64::<BigEndian>()?;
        let block_time = rdr.read_u64::<BigEndian>()?;
        let mut block_hash = [0u8; 32];
        rdr.read_exact(&mut block_hash)?;
        let results_len = rdr.read_u8()?;
        let mut results = Vec::with_capacity(results_len.into());
        for _ in 0..results_len {
            let lamports = rdr.read_u64::<BigEndian>()?;
            let rent_epoch = rdr.read_u64::<BigEndian>()?;
            let executable_u8 = rdr.read_u8()?;
            let executable = executable_u8 != 0;
            let mut owner = [0u8; 32];
            rdr.read_exact(&mut owner)?;
            let data_len = rdr.read_u32::<BigEndian>()?;
            let mut data = vec![0u8; data_len.try_into().unwrap()];
            rdr.read_exact(&mut data)?;
            results.push(SolanaAccountResult {
                lamports,
                rent_epoch,
                executable,
                owner,
                data,
            })
        }
        Ok(SolanaAccountQueryResponse {
            slot_number,
            block_time,
            block_hash,
            results,
        })
    }
}
