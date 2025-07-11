use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    agent::AgentOutput,
    output_parser::{
        extract_from_codeblock, fix_text, flatten_final_answer, is_malformed_event,
        is_malformed_event_str, parse_partial_json, remove_thought, OutputParseError,
    },
    schemas::ToolCall,
    tools::Tool,
    utils::helper::normalize_tool_name,
};

use super::Instructor;

const DEFAULT_TOOL_PROMPT: &str = r#"

<INSTRUCTIONS>
- You have two options:
    1. Use a tool
    2. Give your final answer
- You may repeat tool use cycle as many times as needed before giving your final answer
- When not using a tool, directly give your final answer
- ALL RESPONSES MUST BE IN JSON FORMAT

Option 1 : Use a tool (If you have tools and you need to use them)
The following is the description of the tools available to you:
{{tools}}
- IF YOU DON'T HAVE TOOLS, PASS THIS OPTION

<TOOL_USAGE_OUTPUT_FORMAT>
{
    "action": (string), The action to take; MUST BE one of [{{tool_names}}]
    "action_input": (object), The input to the action, JSON object. The structure object depends on the action you are taking, and is specified in the tool description below.
}
</TOOL_USAGE_OUTPUT_FORMAT>


Option 2 : Give your best final answer
- Only return a final answer once all required tools have been used
- **NEVER RETURN TOOL USE PLAN AS A FINAL ANSWER**

<FINAL_ANSWER_OUTPUT_FORMAT>
{
    "final_answer": Your final answer as requested by the user. The final answer should follow the format specified in the user request
}
</FINAL_ANSWER_OUTPUT_FORMAT>

</INSTRUCTIONS>"#;

const ACTION_KEY: &str = "action";
const ACTION_INPUT_KEY: &str = "action_input";
const FINAL_ANSWER_KEY: &str = "final_answer";
const VALID_KEYS: &[&[&str]] = &[&[ACTION_KEY, ACTION_INPUT_KEY], &[FINAL_ANSWER_KEY]];

#[derive(Default)]
pub struct DefaultInstructor;

impl DefaultInstructor {
    fn value_to_agent_event(&self, value: Value) -> Result<AgentOutput, serde_json::Error> {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum AgentOutputHelp {
            Action {
                #[serde(default)]
                id: Option<String>,
                action: String,
                #[serde(default)]
                action_input: Option<Value>,
            },
            FinalAnswer {
                final_answer: Value,
            },
        }

        let helper: AgentOutputHelp = serde_json::from_value(value)?;
        let agent_output = match helper {
            AgentOutputHelp::Action {
                id,
                action,
                action_input,
            } => AgentOutput::Action(vec![ToolCall {
                id: id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                name: action,
                arguments: action_input.unwrap_or(Value::Null),
            }]),
            AgentOutputHelp::FinalAnswer { final_answer } => {
                AgentOutput::Finish(flatten_final_answer(final_answer)?)
            }
        };

        Ok(agent_output)
    }

    fn parse_with_regex(&self, text: &str) -> Option<AgentOutput> {
        let final_answer_re = Regex::new(r#"(?m)"final_answer"\s*:\s*"(.*)"\s*\n"#).unwrap();
        let action_regex = Regex::new(r#"(?m)"action"\s*:\s*"(.*)"\s*\n"#).unwrap();
        let action_input_regex = Regex::new(r#"(?m)"action_input"\s*:\s*"(.*)"\s*\n"#).unwrap();

        if let Some(final_answer) = final_answer_re.captures(text) {
            let final_answer = final_answer.get(1)?.as_str();
            Some(AgentOutput::Finish(fix_text(final_answer)))
        } else if let (Some(action), Some(action_input)) = (
            action_regex.captures(text),
            action_input_regex.captures(text),
        ) {
            let action = action.get(1)?.as_str();
            let action_input = action_input.get(1)?.as_str();
            Some(AgentOutput::Action(vec![ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: fix_text(action),
                arguments: serde_json::from_str(action_input).ok()?,
            }]))
        } else {
            None
        }
    }
}

impl Instructor for DefaultInstructor {
    fn create_suffix(&self, tools: &[&dyn Tool]) -> String {
        let tool_names = tools
            .iter()
            .map(|tool| normalize_tool_name(&tool.name()))
            .collect::<Vec<_>>()
            .join(", ");
        let tool_string = tools
            .iter()
            .map(|tool| tool.to_plain_description())
            .collect::<Vec<_>>()
            .join("\n");
        DEFAULT_TOOL_PROMPT
            .replace("{{tool_names}}", &tool_names)
            .replace("{{tools}}", &tool_string)
    }

    fn parse_from_text(&self, output: String) -> Result<AgentOutput, OutputParseError> {
        let text = remove_thought(&output);
        let text = extract_from_codeblock(text);

        let json = parse_partial_json(text, false);

        let is_malformed_event = match json.as_ref() {
            Ok(json) => is_malformed_event(json, VALID_KEYS),
            Err(_) => is_malformed_event_str(text, VALID_KEYS),
        };

        match json
            .and_then(|json| self.value_to_agent_event(json))
            .or_else(|e| self.parse_with_regex(text).ok_or(e))
        {
            Ok(agent_event) => Ok(agent_event),
            Err(_) if !is_malformed_event => Ok(AgentOutput::Finish(text.into())),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_parse_agent_output() {
        let test_output = indoc! {r#"
            ```json
            {
                "action": "generate",
                "action_input": "Hello, world!"
            }
            ```
        "#};

        let parsed_output = DefaultInstructor.parse_from_text(test_output.into());

        match parsed_output {
            Ok(AgentOutput::Action(tool_calls)) => {
                assert!(tool_calls.len() == 1);
                let tool_call = &tool_calls[0];
                assert_eq!(tool_call.name, "generate");
                assert_eq!(tool_call.arguments, "Hello, world!");
            }
            _ => panic!("Expected AgentEvent::Action, got {parsed_output:#?}"),
        }

        let test_final_answer = indoc! {r#"
            ```json
            {
                "final_answer": "Goodbye, world!"
            }
            ```
        "#};

        let parsed_output = DefaultInstructor.parse_from_text(test_final_answer.into());

        match parsed_output {
            Ok(AgentOutput::Finish(final_answer)) => {
                assert_eq!(final_answer, "Goodbye, world!");
            }
            _ => panic!("Expected AgentEvent::Finish, got {parsed_output:#?}"),
        }
    }

    #[test]
    fn test_parse_object_answer() {
        let test_final_answer = indoc! {r#"
            ```json
            {
                "final_answer": [
                    {
                        "ingredients": "Universal ANI threshold validation: Established a 95–96% ANI species boundary across 6,787 prokaryotic genomes spanning 22 phyla, supported by empirical evidence of a distinct ANI distribution valley at this value, forming a universal genomic criterion supplanting DDH."
                    },
                    {
                        "ingredients": "Optimized 16S threshold via cross-validation: Derived a 98.65% 16S rRNA sequence similarity threshold for species demarcation through F-score optimization and logarithmic transformation of ANI-16S correlations, enabling alignment with genomic standards and resolving prior linear model discrepancies."
                    },
                    {
                        "ingredients": "Statistical methodology innovation: Introduced a precision-recall framework combining F-score maximization and cross-validation to objectively determine genomic-phenotypic species boundaries, overcoming subjective reliance on historical DDH values."
                    },
                    {
                        "ingredients": "Taxonomic reclassification mandates: Identified inconsistent species classifications (e.g., Bacillus anthracis-cereus, Shigella-E. coli) requiring genomic reevaluation as ANI exceeds 96% despite existing taxonomic separation, necessitating revised microbial nomenclature."
                    },
                    {
                        "ingredients": "16S taxonomic limitations: Revealed intra-species 16S rRNA heterogeneity (up to 9.8% in Halomicrobium) and genus-specific sequencing artifacts undermining solely marker-based classification, necessitating multi-genome analyses for accuracy."
                    },
                    {
                        "ingredients": "Genomic data quality criteria: Established stringent quality controls (>7× 16S sequencing depth, full-genome completion) to eliminate errors from low-coverage drafts (e.g., Neisseria meningitidis exceptions), ensuring valid ANI calculations."
                    },
                    {
                        "ingredients": "Bimodal ANI distribution proof: Empirically validated species boundary via bimodal ANI histograms displaying consistent inter-species valley at 95–96%, confirming universal lineage-independent validity across prokaryotes."
                    }
                ]
            }
            ```
        "#};

        let result = DefaultInstructor.parse_from_text(test_final_answer.into());

        match result {
            Ok(AgentOutput::Finish(final_answer)) => {
                println!("{final_answer}");
            }
            _ => panic!("Expected AgentEvent::Finish, got {result:#?}"),
        }
    }

    #[test]
    fn test_parse_quotation_in_string() {
        let test_final_answer: &str = indoc! {r#"
            ```json
            {
            "final_answer": "\## Thinking process\n\n### Step 1:\n- Proposal Candidate 1: Combining SetA’s `ANI 95-96% as universal boundary\` {A1} with SetB’s `log-transformed linear correlation between ANI and 16S\` {B3} and `statistical statistical framework for threshold validation\` {B5} to unify genomic and 16S classifications while avoiding sampling bias. \n- Proposal Candidate 2: Exploring discrepancies caused by intragenomic 16S variability (SetA4 和 SetB2/4) by integrating heterogeneity analyses {B4}, cross-disciplinary examples {B2}, and statistical rigor {B5} to establish multi-locus metrics for species delineation.\n- Proposal Candidate 3: Improving statistical validation methods using SetA2’s cross-validation and SetB5’s formal statistical framework to create the gold-standard accuracy for genomic/prokaryotic species boundaries.\n- Proposal Candidate 4: Creating quality control protocols by combining SetA5’s genome filtering criteria and SetB6’s sequencing standards to eliminate bias in threshold testing.\n- Proposal Candidate 5: Genomically reclassify extremophiles (SetB7) using ANI data {A1} and adjustment for discrepancies (SetA4) to address taxonomic inconsistencies.\n- Proposal Candidate 6: Evaluating the `hollow at ANI 95-96% {A3} THROUGH SETB에의한 log-linear model {B3} AND 多 factor in technical complexities, such as sequencing errors DISCLAIMED by rigorous quality control {A5 - B6}.\n\n### Step 2:\n- Enhancing Proposal Candidate 1: Incorporating SetB1’s `1 million pairwise comparisons for 16S准确性\` AND statistical precision-recall curves {A2} to ensure ANI/16S Correlation’s robustness across all prokaryotes.\n- Enhancing Proposal Candidate 2: Adding SetA’s `genetic quality metrics\` {A5} AND SetB’s统计" mathematical framework\` to quantify the heterogeneity’s impact on thresholds.\n- Enhancing Proposal Candidate 3: Supplementing with SetB3’s logarithmic analysis to unify non-linear relationships, enabling broader applicability of thresholds AND QUANTIFICATION 统一性.\n- Enhancing Proposal Candidate 5: Using SetB3’s modelo和 log-linear correlations to clarify which borderline cases BEST WARRANT reclassification.\n- Splitting Proposal Candidate 4: Forming独立的 PROMAA4L about data purification ({A5 - B6}) AND the sibling proposal that integrates these with fundamental threshold validation (Proposal1 and 5).\n- Forming PROPS Focused on heterogeneity mechanisms: adding SetB’s `extremophiles’高 rRNA异质性 needing reclassification\` {B7} to discrepancy studies (Proposal2와5) to?>### Step 3:\n- 각 proposal’s 구성 ideas:\n\n1. 자 props1 Enhanced: {A1, A2, A5, B1, B3, B5, B6}\n2. Props2-1 (detailed):(A4, B2, B4, B5, B7)\n3. Props2-2: 同上  but separate - but better to calc each: \n3.@@ PROPOSAL 2 Enhances to Become Two Paths:\n  -  **Path A**: getAddressin异质性 WITHOUT recurrent的 stats得到  {A4, B2, B4}\n  -  **Path B**: Including统计framework得到  {A4, B2， B4， B5}\n Thus needing_average counting but 和 非为例， i'll& proceed ，\n\n但 total the following：:\n\n- **Proposal1**: {A1, A2, A5, B1, B3, B5, B6} → SetA ideas count: A1+A2+A5 →3， SetB ideas:5（B1,B3,B5,B6）의? No: B ideas는 B1, B3, B5, B6 each once →4次.\n- **Proposal2-1**: {A4, B2, B4} → SetA:1（a4）, SetB:3（each）.\n- **Proposal2-2**: {A4, B4, B5} → 추가 يعد 1次 for B5（already counted in P1）.\n可能 it's better to recount with the acceptable grouping.\n\nHere hypothetically consider the following grouped proposals后 enhancement ， stating all used ideas..\n\n\n\n- **Proposal Candidate1-Enhanced**: \"Unifying genomic and phenotypic species boundaries通过 integrated thresholds):\n  - Ideas: A1 (95-96% ANI), B1 (16S threshold), B3 (log-linear correlation), B5 (statistical validation), A2 (cross-val methods), AND A5 (quali control).\n- **Prop2 (Lineage-centric threshold adjustments motivoel)**:\n  - Ideas: A4 (works已经). B2 (ANI vs 16S exceptions), B4 (rRNA variability), B5 (statistical methods).\n- **Proposal3 (Statistical assortment DEFAULT):\n  - **{A2, B5} → Simple 2 ideas.\n- **Proposal4 (Quality protocols):\n  - **{A5, B6} → 2 ideas.\n- **Proposal5 (Extremophile reclassification):\n  - **{A1, A4, B7} → count accordingly.\n- **Proposal6 (hollow ANI time **? perhaps merged into 1, but if seperate:\n   - {A3, B3, A5, B6} → additional A3.\n这样 total counts would be:\n\n### From SetA:\n- A1: used in Constituent1 和5 → 2次\n- A2:在 constituent1 →1次\n- A3:在 constituent6 →1次\n- A4:在 Prop2 and5→ 2次\n- A5:在 constituent1、4、6→ 3次\n- **Total SetA: 2 +1 +1+2+3 =9 ?** \n\n### From SetB,\n- B1: 1次\n- B2:1次\n- B3:在 constituent1和6 →2次\n- B4:1次\n- B5: 在 constituent1 and2, and3, and6 → 더, accounted as３?\n- **总 SetB:\n- B1:1, B2:1; B3:2; B4:1; B5:3； B6:2； B7:1.\n总 =1+1+2+1+3+2+1 =11.\n\nThis requires A precise grouping ，假设 my initial counts在 the absence of. overlapping, this would be the case.\n\n최終:\n\nAfter carefully considering the optimal combinations and reapments of the proposals， the following counts are derived：\n- **From SetA**: The ideas are used a total of **8** times across all proposals. This is because:\n  - A1 é employed in 2 proposals, \n  - A2 in 1, \n  - A3 in 1,\n  - A4 in 2,\n  - A5 in 2.\n  Total: 2+1+1+2+2=8.\n\n- **From SetB**: The total of **9**次使用:\n  - B1:1,\n  - B2:1,\n  - B3:1,\n  - B4:1,\n  - B5:3（appear在 3 proposals）,\n  - B6:2,\n  - B7:1.\n adding together:1+1+1+1+3+2+1=9.\n\nTherefore:\\n\n# From set A = 8\\n# From set B = 9"
            }
            ```
        "#};

        let result = DefaultInstructor.parse_from_text(test_final_answer.into());

        match result {
            Ok(AgentOutput::Finish(final_answer)) => {
                println!("{final_answer}");
            }
            _ => panic!("Expected AgentEvent::Finish, got {result:#?}"),
        }
    }

    #[test]
    fn test_parse_trailing_comma() {
        let test_final_answer: &str = indoc! {r#"
            {
                "final_answer": "Understanding the circadian regulation of sphingosine-1-phosphate (S1P) signaling is crucial for advancing our comprehension of how lipid mediators influence cardiovascular physiology across different times of the day. The research findings suggest that fluctuations in S1P levels, guided by intrinsic circadian rhythms, play a significant role in regulating cardiovascular parameters such as vascular tone and heart rate variability. With the emerging insights into the time-dependent nature of S1P signaling, future studies in cardiovascular physiology can build upon this foundation to investigate how disruptions in these rhythms might contribute to cardiovascular diseases.\n\nThe amalgamation of circadian biology with lipid signalling not only enhances our understanding of the mechanisms underlying cardiovascular homeostasis but also opens new avenues for therapeutic interventions. By elucidating the fundamental principles that govern lipid mediator activity in relation to time, researchers may develop innovative strategies to address cardiovascular health issues that fluctuate with the circadian cycle. This knowledge serves to not only improve our grasp of cardiovascular responses to physiological changes but also enrich the field of chronobiology, which seeks to understand how biological processes are influenced by time. Thus, there lies a tremendous potential for translating these findings into clinical practice, optimising treatment protocols according to the circadian patterns of S1P signaling in individuals, and ultimately fostering advancements in the management of cardiovascular diseases. As research continues, the integration of these insights may significantly contribute to the evolution of both cardiovascular physiology and lipid signalling research, paving the way for comprehensive understanding and novel therapeutic approaches.",
            }
        "#};

        let result = DefaultInstructor.parse_from_text(test_final_answer.into());

        match result {
            Ok(AgentOutput::Finish(final_answer)) => {
                println!("{final_answer}");
            }
            _ => panic!("Expected AgentEvent::Finish, got {result:#?}"),
        }
    }

    #[test]
    fn test_remove_thoughts() {
        let text = indoc! {r#"
            <think> This is a thought </think>
            {
                "action": "generate",
                "action_input": "Hello, world!"
            }
        "#};

        let result = DefaultInstructor.parse_from_text(text.into()).unwrap();
        match result {
            AgentOutput::Action(tool_calls) => {
                assert_eq!(tool_calls.len(), 1);
                let tool_call = &tool_calls[0];
                assert_eq!(tool_call.name, "generate");
                assert_eq!(tool_call.arguments, "Hello, world!");
            }
            _ => panic!("Expected AgentEvent::Action, got {result:#?}"),
        }
    }

    #[test]
    fn test_final_answer_multiline() {
        let text = indoc! {r#"
        {
            "final_answer": "Cyclin-dependent kinases (CDKs) have long been established as critical regulators of the cell cycle, driving cellular progression through distinct phases and presenting attractive targets for oncological intervention [Molecular mechanisms of cell death: recommendations of the Nomenclature Committee on Cell Death 2018](https://doi.org/10.1038/s41418-017-0012-4). Initially developed as cytotoxic agents, CDK inhibitors – including palbociclib, ribociclib, and abemaciclib – have demonstrated significant clinical efficacy in hormone receptor-positive, HER2-negative breast cancer, and are increasingly being investigated in other malignancies [Effects and mechanisms of innate immune molecules on inhibiting nasopharyngeal carcinoma](https://doi.org/10.1097/cm9.0000000000000132). However, growing clinical evidence reveals that the effects of CDK inhibition extend beyond cell cycle arrest, encompassing significant modulation of the host immune landscape. This emerging paradigm suggests that CDK inhibitors may exert both direct and indirect effects on immune cell function, potentially contributing to both therapeutic efficacy and immune-related adverse events.

        The intricate interplay between CDK inhibition and immune function centres, in part, on the modulation of key pro-inflammatory signalling pathways, particularly the nuclear factor kappa B (NF-κB) pathway. NF-κB is a pleiotropic transcription factor central to the regulation of inflammatory responses, controlling the expression of a diverse array of genes involved in immune cell activation, cytokine production, and survival [Mechanisms and functions of p38 MAPK signalling](https://doi.org/10.1042/bj20100323).  Activation of NF-κB is typically triggered by upstream signalling cascades initiated by pattern recognition receptors, such as Toll-like receptors (TLRs), or through cytokine receptor signalling. These signals converge on the IκB kinase (IKK) complex, leading to IκB phosphorylation, ubiquitination, and subsequent degradation, thereby liberating NF-κB to translocate to the nucleus and initiate transcriptional programs. Emerging evidence suggests that CDK inhibition can disrupt this delicate balance, potentially leading to aberrant NF-κB activation. 

        The precise mechanisms linking CDK inhibition to NF-κB activation remain incompletely understood but likely involve complex interactions between multiple signalling pathways. While the precise molecular details are still under investigation, potential mechanisms include altered regulation of upstream kinases (such as RIP1 and IKK) involved in NF-κB activation, and/or modulation of NF-κB transcriptional activity itself. A comprehensive understanding of these mechanisms is crucial not only for elucidating the immunological consequences of CDK inhibitor therapy, but also for developing strategies to mitigate potential immune-related toxicities and maximize therapeutic benefit. Further investigation into the underlying mechanisms is therefore warranted and will be a focus of current research."
        }"#};

        let result = DefaultInstructor.parse_from_text(text.into()).unwrap();
        match result {
            AgentOutput::Finish(final_answer) => {
                println!("{final_answer}");
            }
            _ => panic!("Expected AgentEvent::Finish, got {result:#?}"),
        }
    }

    #[test]
    fn test_final_answer_raw_text() {
        let text = indoc! {"
        My final answer is 5"};

        let result = DefaultInstructor.parse_from_text(text.into()).unwrap();

        match result {
            AgentOutput::Finish(final_answer) => assert_eq!(final_answer, "My final answer is 5"),
            _ => panic!("Expected AgentEvent::Finish, got {result:#?}"),
        }
    }

    #[test]
    fn test_final_answer_raw_json() {
        let text = indoc! {r#"
            ```json
            [
                {
                    "Title": "Wavelength-selective cleavage of photoprotecting groups: strategies and applications in dynamic systems",
                    "URL": "https://doi.org/10.1039/c5cs00118h",
                    "Explanation": "This article reviews strategies for wavelength-selective deprotection of photoprotecting groups, which is directly relevant to the proposal's use of photolabile protecting groups (PPGs). It discusses the design and application of these groups for controlling functions in dynamic systems, offering insights into optimizing the orthogonality and efficiency of the dual-PPG system proposed for spatiotemporally controlled Plk1 inhibition. The article highlights the potential for using different wavelengths to control different processes, which could be valuable for fine-tuning the activation of the dual-PPG system."
                },
                {
                    "Title": "Illuminating the Chemistry of Life: Design, Synthesis, and Applications of “Caged” and Related Photoresponsive Compounds",
                    "URL": "https://doi.org/10.1021/cb900036s",
                    "Explanation": "This review provides a comprehensive overview of caged compounds and other photoresponsive reagents used in biological research. It discusses the challenges associated with their design, synthesis, and application, as well as recent advances in the field. This is highly relevant to the proposal, as it provides a broader context for the use of photolabile protecting groups and highlights the importance of precise control over spatial and temporal activation of biological processes. It also addresses the limitations of current instrumentation and potential strategies for overcoming them."
                },
                {
                    "Title": "Mitochondrial pharmacology",
                    "URL": "https://doi.org/10.1016/j.tips.2012.03.010",
                    "Explanation": "While not directly focused on photochemistry, this review on mitochondrial pharmacology is relevant because it discusses strategies for targeted drug delivery and modulation of cellular processes. The proposal aims to achieve precise spatiotemporal control of Plk1 inhibition, and this article provides insights into the broader challenges of delivering and activating drugs within specific cellular compartments. Understanding the principles of mitochondrial pharmacology could inform the design of the dual-PPG system to enhance its specificity and efficacy."
                }
            ]
            ```"#};

        let result = DefaultInstructor.parse_from_text(text.into()).unwrap();

        match result {
            AgentOutput::Finish(final_answer) => {
                println!("{final_answer}");
            }
            _ => panic!("Expected AgentEvent::Finish, got {result:#?}"),
        }
    }

    #[test]
    fn test_final_answer_malformed_json() {
        let text = r#"
        {
            "final_answer": {
                "lalala",
                "foo": "bar",
                "baz": 42,
            }
        }
        "#;

        let result = DefaultInstructor.parse_from_text(text.into());

        assert!(result.is_err(), "Expected err, got {result:#?}");
    }

    #[test]
    fn test_parse_list() {
        let text = r#"["`hypoxia` AND `endothelial mitotic activity` AND `vascular remodeling` AND `cellular response`", "`regulatory motifs` AND `hypoxia` AND `gene regulation` AND `DNA binding`", "`vascular responses` AND `genomic data` AND `hypoxia` AND `molecular mechanisms`"]
"#;

        let result = DefaultInstructor.parse_from_text(text.into());

        println!("{result:#?}");
    }
}
