use super::{CasperNodePorts, NodeState};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, not_line_ending, space1},
    combinator::map,
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

pub fn parse_node_state(input: &str) -> IResult<&str, NodeState> {
    alt((
        map(tag("RUNNING"), |_| NodeState::Running),
        map(tag("STOPPED"), |_| NodeState::Stopped),
    ))(input)
}

pub fn parse_cctl_infra_net_start_line(input: &str) -> IResult<&str, (u8, u8, NodeState)> {
    let (remainder, (_, _, group_id, _, node_id, _, status, _)) = tuple((
        multispace0,
        tag("validator-group-"),
        nom::character::complete::u8,
        tag(":cctl-node-"),
        nom::character::complete::u8,
        space1,
        parse_node_state,
        not_line_ending,
    ))(input)?;

    Ok((remainder, (group_id, node_id, status)))
}

pub fn parse_cctl_infra_net_start_lines(input: &str) -> IResult<&str, Vec<(u8, u8, NodeState)>> {
    let (remainder, _) = nom::bytes::complete::take_until("validator-group")(input)?;
    separated_list0(tag("\n"), parse_cctl_infra_net_start_line)(remainder)
}

pub fn parse_cctl_infra_node_view_port_line(input: &str) -> IResult<&str, (u8, CasperNodePorts)> {
    let (
        remainder,
        (
            _,
            _,
            group_id,
            _,
            consensus_port,
            _,
            rpc_port,
            _,
            rest_port,
            _,
            sse_port,
            _,
            speculative_exec_port,
            _,
        ),
    ) = tuple((
        nom::bytes::complete::take_until("node-"),
        tag("node-"),
        nom::character::complete::u8,
        tag(" -> CONSENSUS @ "),
        nom::character::complete::u16,
        tag(" :: RPC @ "),
        nom::character::complete::u16,
        tag(" :: REST @ "),
        nom::character::complete::u16,
        tag(" :: SSE @ "),
        nom::character::complete::u16,
        tag(" :: SPECULATIVE_EXEC @ "),
        nom::character::complete::u16,
        not_line_ending,
    ))(input)?;

    Ok((
        remainder,
        (
            group_id,
            CasperNodePorts {
                consensus_port,
                rpc_port,
                rest_port,
                sse_port,
                speculative_exec_port,
            },
        ),
    ))
}

pub fn parse_cctl_infra_node_view_port_lines(
    input: &str,
) -> IResult<&str, Vec<(u8, CasperNodePorts)>> {
    separated_list0(tag("\n"), parse_cctl_infra_node_view_port_line)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    #[test]
    fn test_parse_cctl_infra_net_start_line() -> Result<(), Error> {
        let input = "validator-group-1:cctl-node-1    RUNNING   pid 428229, uptime 0:09:06\n";
        let (_, parsed) = parse_cctl_infra_net_start_line(input)?;
        Ok(assert_eq!((1, 1, NodeState::Running), parsed))
    }
    #[test]
    fn test_parse_cctl_infra_net_start_lines() -> Result<(), Error> {
        let input = r#"
            2024-02-06T14:06:43.332420 [INFO] [431123] CCTL :: network spin up begins ... please wait
            2024-02-06T14:06:43.334599 [INFO] [431123] CCTL :: ... starting network
            2024-02-06T14:06:43.337054 [INFO] [431123] CCTL :: ... ... genesis bootstrap nodes
            2024-02-06T14:06:44.445527 [INFO] [431123] CCTL :: ... ... genesis non-bootstrap nodes
            validator-group-1:cctl-node-1    RUNNING   pid 428229, uptime 0:09:06
            validator-group-1:cctl-node-2    RUNNING   pid 428230, uptime 0:09:06
            validator-group-1:cctl-node-3    RUNNING   pid 428231, uptime 0:09:06
            validator-group-2:cctl-node-4    RUNNING   pid 428296, uptime 0:09:05
            validator-group-2:cctl-node-5    RUNNING   pid 428297, uptime 0:09:05
            validator-group-3:cctl-node-10   STOPPED   Not started
            validator-group-3:cctl-node-6    STOPPED   Not started
            validator-group-3:cctl-node-7    STOPPED   Not started
            validator-group-3:cctl-node-8    STOPPED   Not started
            validator-group-3:cctl-node-9    STOPPED   Not started
        "#;
        let (_, parsed) = parse_cctl_infra_net_start_lines(input)?;
        let expected = vec![
            (1, 1, NodeState::Running),
            (1, 2, NodeState::Running),
            (1, 3, NodeState::Running),
            (2, 4, NodeState::Running),
            (2, 5, NodeState::Running),
            (3, 10, NodeState::Stopped),
            (3, 6, NodeState::Stopped),
            (3, 7, NodeState::Stopped),
            (3, 8, NodeState::Stopped),
            (3, 9, NodeState::Stopped),
        ];
        Ok(assert_eq!(expected, parsed))
    }
    #[test]
    fn test_parse_cctl_infra_node_view_port_line() -> Result<(), Error> {
        let input = "2024-02-06T17:28:10.731821 [INFO] [514427] CCTL :: node-1 -> CONSENSUS @ 22101 :: RPC @ 11101 :: REST @ 14101 :: SSE @ 18101 :: SPECULATIVE_EXEC @ 25101";
        let (_, parsed) = parse_cctl_infra_node_view_port_line(input)?;
        Ok(assert_eq!(
            (
                1,
                CasperNodePorts {
                    consensus_port: 22101,
                    rpc_port: 11101,
                    rest_port: 14101,
                    sse_port: 18101,
                    speculative_exec_port: 25101
                }
            ),
            parsed
        ))
    }
    #[test]
    fn test_parse_cctl_infra_node_view_port_lines() -> Result<(), Error> {
        let input = r#"
            2024-02-06T17:28:10.728367 [INFO] [514427] CCTL :: ------------------------------------------------------------------------------------------------------
            2024-02-06T17:28:10.731821 [INFO] [514427] CCTL :: node-1 -> CONSENSUS @ 22101 :: RPC @ 11101 :: REST @ 14101 :: SSE @ 18101 :: SPECULATIVE_EXEC @ 25101
            2024-02-06T17:28:10.732997 [INFO] [514427] CCTL :: ------------------------------------------------------------------------------------------------------
            2024-02-06T17:28:10.737211 [INFO] [514427] CCTL :: node-2 -> CONSENSUS @ 22102 :: RPC @ 11102 :: REST @ 14102 :: SSE @ 18102 :: SPECULATIVE_EXEC @ 25102
            2024-02-06T17:28:10.738952 [INFO] [514427] CCTL :: ------------------------------------------------------------------------------------------------------
            2024-02-06T17:28:10.742946 [INFO] [514427] CCTL :: node-3 -> CONSENSUS @ 22103 :: RPC @ 11103 :: REST @ 14103 :: SSE @ 18103 :: SPECULATIVE_EXEC @ 25103
            2024-02-06T17:28:10.744218 [INFO] [514427] CCTL :: ------------------------------------------------------------------------------------------------------
            2024-02-06T17:28:10.748632 [INFO] [514427] CCTL :: node-4 -> CONSENSUS @ 22104 :: RPC @ 11104 :: REST @ 14104 :: SSE @ 18104 :: SPECULATIVE_EXEC @ 25104
            2024-02-06T17:28:10.749922 [INFO] [514427] CCTL :: ------------------------------------------------------------------------------------------------------
            2024-02-06T17:28:10.754162 [INFO] [514427] CCTL :: node-5 -> CONSENSUS @ 22105 :: RPC @ 11105 :: REST @ 14105 :: SSE @ 18105 :: SPECULATIVE_EXEC @ 25105
            2024-02-06T17:28:10.755567 [INFO] [514427] CCTL :: ------------------------------------------------------------------------------------------------------
        "#;
        let (_, parsed) = parse_cctl_infra_node_view_port_lines(input)?;
        let expected = vec![
            (
                1,
                CasperNodePorts {
                    consensus_port: 22101,
                    rpc_port: 11101,
                    rest_port: 14101,
                    sse_port: 18101,
                    speculative_exec_port: 25101,
                },
            ),
            (
                2,
                CasperNodePorts {
                    consensus_port: 22102,
                    rpc_port: 11102,
                    rest_port: 14102,
                    sse_port: 18102,
                    speculative_exec_port: 25102,
                },
            ),
            (
                3,
                CasperNodePorts {
                    consensus_port: 22103,
                    rpc_port: 11103,
                    rest_port: 14103,
                    sse_port: 18103,
                    speculative_exec_port: 25103,
                },
            ),
            (
                4,
                CasperNodePorts {
                    consensus_port: 22104,
                    rpc_port: 11104,
                    rest_port: 14104,
                    sse_port: 18104,
                    speculative_exec_port: 25104,
                },
            ),
            (
                5,
                CasperNodePorts {
                    consensus_port: 22105,
                    rpc_port: 11105,
                    rest_port: 14105,
                    sse_port: 18105,
                    speculative_exec_port: 25105,
                },
            ),
        ];
        Ok(assert_eq!(expected, parsed))
    }
}
