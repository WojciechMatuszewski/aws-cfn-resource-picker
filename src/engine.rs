use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

pub struct Engine {
    client: aws_sdk_cloudformation::Client,
}

impl Engine {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_cloudformation::Client::new(&config);

        return Engine { client: client };
    }
}

pub struct Stacks(Vec<String>);

impl Into<String> for Stacks {
    fn into(self) -> String {
        return self.0.join("\n");
    }
}

#[async_trait]
pub trait StacksGetter {
    async fn get_stacks(&self) -> Result<Stacks>;
}

#[async_trait]
impl StacksGetter for Engine {
    async fn get_stacks(&self) -> Result<Stacks> {
        let result = self
            .client
            .list_stacks()
            .stack_status_filter(aws_sdk_cloudformation::model::StackStatus::CreateComplete)
            .stack_status_filter(aws_sdk_cloudformation::model::StackStatus::UpdateComplete)
            .send()
            .await?;

        let stack_summaries = result
            .stack_summaries()
            .expect("Failed to get the stack summaries");

        let stack_names: Vec<String> = stack_summaries
            .into_iter()
            .map(|summary| {
                return summary
                    .stack_name()
                    .expect("Failed to get the stack name")
                    .to_owned();
            })
            .collect();

        return Ok(Stacks(stack_names));
    }
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub resource_physical_id: String,
    pub resource_type: String,
}

impl Into<String> for Resource {
    fn into(self) -> String {
        return format!("{} ({})", self.resource_physical_id, self.resource_type);
    }
}

impl TryFrom<String> for Resource {
    type Error = anyhow::Error;

    fn try_from(resource: String) -> Result<Self> {
        let resource = resource.replace(&['(', ')'][..], "");
        let resource_parts = resource.split(" ").collect::<Vec<&str>>();
        if resource_parts.len() < 2 {
            return Err(anyhow!("Malformed resource str"));
        }

        let resource_physical_id = match resource_parts.get(0) {
            Some(logical_id) => logical_id.to_string(),
            None => return Err(anyhow!("Failed")),
        };

        let resource_type = match resource_parts.get(1) {
            Some(resource_type) => resource_type.to_string(),
            None => return Err(anyhow!("Failed")),
        };

        return Ok(Self {
            resource_physical_id,
            resource_type,
        });
    }
}

impl Resource {
    pub fn to_console_url_path(&self, _stack_name: &str) -> Result<String> {
        let path = match self.resource_type.as_str() {
            "AWS::Lambda::Function" => {
                format!(
                    "lambda/home#/functions/{}?tab=code",
                    self.resource_physical_id
                )
            }
            "AWS::S3::Bucket" => {
                format!("s3/buckets/{}?tab=objects", self.resource_physical_id)
            }
            "AWS::DynamoDB::Table" => {
                format!("dynamodbv2/home#table?name={}", self.resource_physical_id)
            }
            _ => return Err(anyhow!("Not mapped!")),
        };

        return Ok(path);
    }
}

#[derive(Debug, Clone)]
pub struct Resources(Vec<Resource>);

impl FromIterator<Resource> for Resources {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Resource>,
    {
        let mut resources: Vec<Resource> = vec![];
        for resource in iter {
            resources.push(resource)
        }

        return Self(resources);
    }
}

impl Into<String> for Resources {
    fn into(self) -> String {
        return self
            .0
            .iter()
            .map(|resource| {
                return resource.to_owned().into();
            })
            .collect::<Vec<String>>()
            .join("\n");
    }
}

#[async_trait]
pub trait ResourceGetter {
    async fn get_resources(&self, stack_name: &str) -> Result<Resources>;
}

#[async_trait]
impl ResourceGetter for Engine {
    async fn get_resources(&self, stack_name: &str) -> Result<Resources> {
        let result = self
            .client
            .list_stack_resources()
            .stack_name(stack_name)
            .send()
            .await
            .context("Failed to fetch stack resources")?;

        let stack_resource_summaries = result
            .stack_resource_summaries()
            .expect("Failed to get stack resource summaries");

        let resources: Resources = stack_resource_summaries
            .into_iter()
            .map(|summary| {
                let resource_type = summary
                    .resource_type()
                    .expect("Failed to extract resource type")
                    .to_owned();

                let resource_physical_id = summary
                    .physical_resource_id()
                    .expect("Failed to get the resource physical id")
                    .to_owned();

                return Resource {
                    resource_physical_id,
                    resource_type,
                };
            })
            .collect();

        return Ok(resources);
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use anyhow::Result;

    use super::{Resource, Resources};

    #[test]
    fn resource_from_correct_string() -> Result<()> {
        let resource_string = String::from("PutFunctionRole (AWS::IAM::Role)");
        let resource: Resource = resource_string.try_into()?;

        assert_eq!(resource.resource_physical_id, "PutFunctionRole");
        assert_eq!(resource.resource_type, "AWS::IAM::Role");

        return Ok(());
    }

    #[test]
    fn resource_from_malformed_string() {
        let malformed_string = String::from("foo");
        let resource: Result<Resource> = malformed_string.try_into();

        assert_eq!(resource.is_err(), true);
    }

    #[test]
    fn resources_to_string() {
        let resources_vec: Vec<Resource> = vec![
            Resource {
                resource_physical_id: "first_resource_physical_id".to_string(),
                resource_type: "first_resource_type".to_string(),
            },
            Resource {
                resource_physical_id: "second_resource_physical_id".to_string(),
                resource_type: "second_resource_type".to_string(),
            },
        ];

        let resources = Resources(resources_vec);
        let resources_string: String = resources.into();

        assert_eq!(resources_string, "first_resource_logical_id (first_resource_type)\nsecond_resource_logical_id (second_resource_type)");
    }
}
