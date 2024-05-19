#[cfg(feature = "ssr")]
pub mod image_filter {
    pub async fn prepare_filtered_query(
        tag_filter: Option<(String, Vec<String>)>,
        people_filter: Option<(String, Vec<i64>)>,
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut conditions: Vec<String> = Vec::new();
        let mut joins: Vec<String> = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some((filter_type, tags)) = &tag_filter {
            if !tags.is_empty() {
                let valid_tags: Vec<String> = tags
                    .iter()
                    .filter_map(|tag| {
                        let trimmed_tag = tag.trim().to_lowercase();
                        let is_valid = !trimmed_tag.is_empty()
                            && trimmed_tag.chars().all(|c| {
                                c.is_alphabetic() || c.is_numeric() || c == '-' || c == '_'
                            });
                        if is_valid {
                            Some(trimmed_tag)
                        } else {
                            None
                        }
                    })
                    .collect();

                if !valid_tags.is_empty() {
                    joins.push("LEFT JOIN tagFile tf ON f.id = tf.fileID LEFT JOIN tags t ON tf.tagString = t.tagString".to_string());

                    match filter_type.as_str() {
                        "HAS" => {
                            conditions.push(format!(
                                "t.tagString IN ({})",
                                valid_tags.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                            ));
                            binds.extend(valid_tags);
                        }
                        "NOT" => {
                            conditions.push(format!(
                                "f.id NOT IN (SELECT tf.fileID FROM tagFile tf WHERE tf.tagString IN ({}))",
                                valid_tags.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                            ));
                            binds.extend(valid_tags);
                        }
                        "ONLY" => {
                            let num_tags = valid_tags.len();
                            conditions.push(format!(
                                "(SELECT COUNT(DISTINCT tf.tagString) FROM tagFile tf WHERE tf.fileID = f.id) = {} AND NOT EXISTS (SELECT 1 FROM tagFile tf WHERE tf.fileID = f.id AND tf.tagString NOT IN ({}))",
                                num_tags,
                                valid_tags.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                            ));
                            binds.extend(valid_tags.clone());
                            binds.extend(valid_tags);
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some((filter_type, user_ids)) = &people_filter {
            if !user_ids.is_empty() {
                let valid_ids: Vec<i64> = user_ids
                    .iter()
                    .filter_map(|id| if *id > 0 { Some(*id) } else { None })
                    .collect();

                if !valid_ids.is_empty() {
                    joins.push("LEFT JOIN userFile uf ON f.id = uf.fileID".to_string());

                    match filter_type.as_str() {
                        "HAS" => {
                            conditions.push(format!(
                                "uf.userID IN ({})",
                                valid_ids
                                    .iter()
                                    .map(|id| id.to_string())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            ));
                            binds.extend(valid_ids.iter().map(|id| id.to_string()));
                        }
                        "NOT" => {
                            conditions.push(format!(
                                "f.id NOT IN (SELECT uf.fileID FROM userFile uf WHERE uf.userID IN ({}))",                                valid_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                            ));
                            binds.extend(valid_ids.iter().map(|id| id.to_string()));
                        }
                        "ONLY" => {
                            let num_people = valid_ids.len();
                            conditions.push(format!(
                                "(SELECT COUNT(DISTINCT uf.userID) FROM userFile uf WHERE uf.fileID = f.id) = {} AND NOT EXISTS (SELECT 1 FROM userFile uf WHERE uf.fileID = f.id AND uf.userID NOT IN ({}))",
                                num_people,
                                valid_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                            ));
                            binds.extend(valid_ids.iter().map(|id| id.to_string()));
                            binds.extend(valid_ids.iter().map(|id| id.to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }

        (conditions, joins, binds)
    }

    pub async fn prepare_filtered_query_with_pagination(
        tag_filter: Option<(String, Vec<String>)>,
        people_filter: Option<(String, Vec<i64>)>,
        limit: usize,
        offset: usize,
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let (conditions, joins, mut binds) =
            prepare_filtered_query(tag_filter, people_filter).await;
        binds.push(limit.to_string());
        binds.push(offset.to_string());

        (conditions, joins, binds)
    }

    pub fn build_filtered_query(
        base_query: String,
        conditions: Vec<String>,
        joins: Vec<String>,
        order_by: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> String {
        let mut query = base_query;

        if !joins.is_empty() {
            query.push_str(" ");
            query.push_str(&joins.join(" "));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        if let Some(order) = order_by {
            query.push_str(" ORDER BY ");
            query.push_str(&order);
        }

        if let Some(limit) = limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }

        if let Some(offset) = offset {
            query.push_str(" OFFSET ");
            query.push_str(&offset.to_string());
        }

        query.push_str(";");

        query
    }
}
