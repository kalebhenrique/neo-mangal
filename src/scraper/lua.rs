use crate::domain::manga::{Chapter, Manga, Page};
use crate::error::{NeoError, Result};
use mlua::{Lua, Table, Value};
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;

pub struct LuaScraper {
    pub name: String,
    lua_code: String,
}

impl LuaScraper {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let code = std::fs::read_to_string(path)?;
        Ok(Self {
            name,
            lua_code: code,
        })
    }

    #[allow(dead_code)]
    pub fn from_string(name: &str, code: &str) -> Self {
        Self {
            name: name.to_string(),
            lua_code: code.to_string(),
        }
    }

    fn init_lua(&self) -> Result<Lua> {
        let lua = Lua::new();
        setup_lua_env(&lua)?;
        lua.load(&self.lua_code)
            .exec()
            .map_err(|e| NeoError::Parse(format!("Lua compile error in {}: {}", self.name, e)))?;
        Ok(lua)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Manga>> {
        let lua = self.init_lua()?;
        let search_fn: mlua::Function = lua.globals().get("SearchManga").map_err(|e| {
            NeoError::Parse(format!("SearchManga not found in {}: {}", self.name, e))
        })?;

        let res: Table = search_fn
            .call(query)
            .map_err(|e| NeoError::Other(format!("SearchManga failed in {}: {}", self.name, e)))?;

        let mut mangas = Vec::new();
        for pair in res.sequence_values::<Table>() {
            if let Ok(t) = pair {
                let name: String = t.get("name").unwrap_or_default();
                let clean_name = name
                    .lines()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                let url: String = t.get("url").unwrap_or_default();
                let cover: Option<String> = t.get("cover").ok();
                if !clean_name.is_empty() && !url.is_empty() {
                    mangas.push(Manga {
                        id: url.clone(),
                        title: clean_name,
                        url,
                        cover_url: cover,
                        provider: self.name.clone(),
                    });
                }
            }
        }
        Ok(mangas)
    }

    pub fn chapters(&self, manga_url: &str) -> Result<Vec<Chapter>> {
        let lua = self.init_lua()?;
        let chapters_fn: mlua::Function = lua.globals().get("MangaChapters").map_err(|e| {
            NeoError::Parse(format!("MangaChapters not found in {}: {}", self.name, e))
        })?;

        let res: Table = chapters_fn.call(manga_url).map_err(|e| {
            NeoError::Other(format!("MangaChapters failed in {}: {}", self.name, e))
        })?;

        let mut chapters = Vec::new();
        for pair in res.sequence_values::<Table>() {
            if let Ok(t) = pair {
                let name: String = t.get("name").unwrap_or_default();
                let clean_name =
                    crate::domain::favorite::FavoriteManager::clean_chapter_title(&name);
                let url: String = t.get("url").unwrap_or_default();
                if !clean_name.is_empty() && !url.is_empty() {
                    let number = t
                        .get::<f32>("number")
                        .ok()
                        .or_else(|| {
                            t.get::<String>("number")
                                .ok()
                                .and_then(|s| s.parse::<f32>().ok())
                        })
                        .or_else(|| Chapter::parse_number_from_title(&clean_name))
                        .or_else(|| Chapter::parse_number_from_title(&name));

                    chapters.push(Chapter {
                        id: url.clone(),
                        manga_id: manga_url.to_string(),
                        title: clean_name,
                        url,
                        number,
                    });
                }
            }
        }
        Ok(chapters)
    }

    pub fn pages(&self, chapter_url: &str) -> Result<Vec<Page>> {
        let lua = self.init_lua()?;
        let pages_fn: mlua::Function = lua.globals().get("ChapterPages").map_err(|e| {
            NeoError::Parse(format!("ChapterPages not found in {}: {}", self.name, e))
        })?;

        let res: Table = pages_fn
            .call(chapter_url)
            .map_err(|e| NeoError::Other(format!("ChapterPages failed in {}: {}", self.name, e)))?;

        let mut pages = Vec::new();
        for pair in res.sequence_values::<Table>() {
            if let Ok(t) = pair {
                let url: String = t.get("url").unwrap_or_default();
                let index: usize = t.get("index").unwrap_or(pages.len());
                if !url.is_empty() {
                    pages.push(Page { index, url });
                }
            }
        }
        Ok(pages)
    }
}

pub fn setup_lua_env(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // 1. Global Helper Functions (Reverse, trim, basename)
    lua.load(
        r#"
        function Reverse(t)
            if not t then return {} end
            local n = #t
            for i = 1, math.floor(n / 2) do
                t[i], t[n - i + 1] = t[n - i + 1], t[i]
            end
            return t
        end

        function trim(s)
            if not s then return "" end
            return (s:gsub("^%s*(.-)%s*$", "%1"))
        end

        function basename(path)
            if not path then return "" end
            return path:match("([^/]+)$") or path
        end
        "#,
    )
    .exec()
    .map_err(|e| NeoError::Parse(e.to_string()))?;

    // 2. HTTP Module
    let http_mod = lua
        .create_table()
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    http_mod
        .set(
            "request",
            lua.create_function(|lua, (method, url): (String, String)| {
                let t = lua.create_table()?;
                t.set("method", method)?;
                t.set("url", url)?;
                t.set("headers", lua.create_table()?)?;
                Ok(t)
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    http_mod
        .set(
            "client",
            lua.create_function(|lua, ()| {
                let c_table = lua.create_table()?;
                c_table.set(
                    "do_request",
                    lua.create_function(|lua, (_self, req_val): (Value, Value)| {
                        let (method, url, headers) = match req_val {
                            Value::String(s) => ("GET".to_string(), s.to_str()?.to_string(), None),
                            Value::Table(t) => {
                                let m = t.get::<String>("method").unwrap_or_else(|_| "GET".to_string());
                                let u = t.get::<String>("url")?;
                                let h = t.get::<Table>("headers").ok();
                                (m, u, h)
                            }
                            _ => return Err(mlua::Error::runtime("Invalid request argument")),
                        };

                        let client = reqwest::blocking::Client::builder()
                            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
                            .timeout(Duration::from_secs(30))
                            .build()
                            .map_err(|e| mlua::Error::runtime(e.to_string()))?;

                        let mut builder = match method.to_uppercase().as_str() {
                            "POST" => client.post(&url),
                            "PUT" => client.put(&url),
                            "DELETE" => client.delete(&url),
                            _ => client.get(&url),
                        };

                        if let Some(h) = headers {
                            for pair in h.pairs::<String, String>() {
                                if let Ok((k, v)) = pair {
                                    builder = builder.header(k, v);
                                }
                            }
                        }

                        let resp = builder.send().map_err(|e| mlua::Error::runtime(e.to_string()))?;
                        let status = resp.status().as_u16();
                        let body = resp.text().map_err(|e| mlua::Error::runtime(e.to_string()))?;

                        let res_table = lua.create_table()?;
                        res_table.set("status", status)?;
                        res_table.set("body", body)?;
                        Ok(res_table)
                    })?,
                )?;
                Ok(c_table)
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    // 3. HTML Module
    let html_mod = lua
        .create_table()
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    html_mod
        .set(
            "parse",
            lua.create_function(|lua, body: String| create_html_doc(lua, &body))
                .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    // 4. HTTP Util Module
    let http_util_mod = lua
        .create_table()
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    http_util_mod
        .set(
            "query_escape",
            lua.create_function(|_lua, s: String| Ok(urlencoding::encode(&s).to_string()))
                .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    http_util_mod
        .set(
            "query_unescape",
            lua.create_function(|_lua, s: String| {
                let decoded = urlencoding::decode(&s).map(|c| c.into_owned()).unwrap_or(s);
                Ok(decoded)
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    // 5. JSON Module
    let json_mod = lua
        .create_table()
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    json_mod
        .set(
            "decode",
            lua.create_function(|lua, s: String| {
                let parsed: serde_json::Value =
                    serde_json::from_str(&s).map_err(|e| mlua::Error::runtime(e.to_string()))?;
                json_to_lua(lua, parsed)
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    json_mod
        .set(
            "encode",
            lua.create_function(|_lua, val: Value| {
                let j = lua_to_json(val);
                let s =
                    serde_json::to_string(&j).map_err(|e| mlua::Error::runtime(e.to_string()))?;
                Ok(s)
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    // 6. Time Module
    let time_mod = lua
        .create_table()
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    time_mod
        .set(
            "sleep",
            lua.create_function(|_lua, secs: f64| {
                std::thread::sleep(Duration::from_secs_f64(secs.max(0.0)));
                Ok(())
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    // 7. Strings Module
    let strings_mod = lua
        .create_table()
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    strings_mod
        .set(
            "trim",
            lua.create_function(|_lua, s: String| Ok(s.trim().to_string()))
                .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    strings_mod
        .set(
            "split",
            lua.create_function(|lua, (s, sep): (String, String)| {
                let t = lua.create_table()?;
                for (i, part) in s.split(&sep).enumerate() {
                    t.set(i + 1, part.to_string())?;
                }
                Ok(t)
            })
            .map_err(|e| NeoError::Parse(e.to_string()))?,
        )
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    // Register modules as globals and preload
    globals
        .set("Http", http_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("http", http_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("Html", html_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("html", html_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("HttpUtil", http_util_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("http_util", http_util_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("Json", json_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("json", json_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("Time", time_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("time", time_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("Strings", strings_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    globals
        .set("strings", strings_mod.clone())
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    let package: Table = globals
        .get("package")
        .map_err(|e| NeoError::Parse(e.to_string()))?;
    let preload: Table = package
        .get("preload")
        .map_err(|e| NeoError::Parse(e.to_string()))?;

    preload
        .set(
            "http",
            lua.create_function(move |_lua, ()| Ok(http_mod.clone()))
                .unwrap(),
        )
        .unwrap();
    preload
        .set(
            "html",
            lua.create_function(move |_lua, ()| Ok(html_mod.clone()))
                .unwrap(),
        )
        .unwrap();
    preload
        .set(
            "http_util",
            lua.create_function(move |_lua, ()| Ok(http_util_mod.clone()))
                .unwrap(),
        )
        .unwrap();
    preload
        .set(
            "json",
            lua.create_function(move |_lua, ()| Ok(json_mod.clone()))
                .unwrap(),
        )
        .unwrap();
    preload
        .set(
            "time",
            lua.create_function(move |_lua, ()| Ok(time_mod.clone()))
                .unwrap(),
        )
        .unwrap();
    preload
        .set(
            "strings",
            lua.create_function(move |_lua, ()| Ok(strings_mod.clone()))
                .unwrap(),
        )
        .unwrap();

    Ok(())
}

fn json_to_lua(lua: &Lua, val: serde_json::Value) -> mlua::Result<Value> {
    match val {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else {
                Ok(Value::Number(n.as_f64().unwrap_or(0.0)))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(&s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map {
                table.set(k, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn lua_to_json(val: Value) -> serde_json::Value {
    match val {
        Value::Nil => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(b),
        Value::Integer(i) => serde_json::json!(i),
        Value::Number(n) => serde_json::json!(n),
        Value::String(s) => {
            serde_json::Value::String(s.to_str().map(|s| s.to_string()).unwrap_or_default())
        }
        Value::Table(t) => {
            let len = t.len().unwrap_or(0);
            if len > 0 {
                let mut vec = Vec::new();
                for v in t.sequence_values::<Value>() {
                    if let Ok(val) = v {
                        vec.push(lua_to_json(val));
                    }
                }
                serde_json::Value::Array(vec)
            } else {
                let mut map = serde_json::Map::new();
                for pair in t.pairs::<String, Value>() {
                    if let Ok((k, v)) = pair {
                        map.insert(k, lua_to_json(v));
                    }
                }
                serde_json::Value::Object(map)
            }
        }
        _ => serde_json::Value::Null,
    }
}

fn create_html_doc(lua: &Lua, html_str: &str) -> mlua::Result<Table> {
    let doc_table = lua.create_table()?;
    let doc_arc = Arc::new(Html::parse_document(html_str));

    let doc_clone = doc_arc.clone();
    doc_table.set(
        "find",
        lua.create_function(move |lua, (_self, selector_str): (Value, String)| {
            query_elements(lua, &doc_clone, &selector_str)
        })?,
    )?;

    Ok(doc_table)
}

fn create_html_element(
    lua: &Lua,
    text_content: String,
    html_content: String,
    attributes: Vec<(String, String)>,
) -> mlua::Result<Table> {
    let el_table = lua.create_table()?;

    let t_clone = text_content;
    el_table.set(
        "text",
        lua.create_function(move |_lua, _self: Value| Ok(t_clone.clone()))?,
    )?;

    let h_clone = html_content.clone();
    el_table.set(
        "html",
        lua.create_function(move |_lua, _self: Value| Ok(h_clone.clone()))?,
    )?;

    let attrs = Arc::new(attributes);
    let attrs_1 = attrs.clone();
    let attr_fn = lua.create_function(move |_lua, (_self, name): (Value, String)| {
        for (k, v) in attrs_1.iter() {
            if k.eq_ignore_ascii_case(&name) {
                return Ok(v.clone());
            }
        }
        Ok(String::new())
    })?;
    el_table.set("attr", attr_fn.clone())?;
    el_table.set("attribute", attr_fn)?;

    // Sub-query find inside element
    let html_sub = html_content;
    el_table.set(
        "find",
        lua.create_function(move |lua, (_self, sel): (Value, String)| {
            let sub_doc = Html::parse_fragment(&html_sub);
            query_elements(lua, &sub_doc, &sel)
        })?,
    )?;

    Ok(el_table)
}

#[allow(dead_code)]
fn create_dummy_element(lua: &Lua) -> mlua::Result<Table> {
    create_html_element(lua, String::new(), String::new(), vec![])
}

fn query_elements(lua: &Lua, html: &Html, selector_str: &str) -> mlua::Result<Table> {
    let selection_table = lua.create_table()?;

    let selector = match Selector::parse(selector_str) {
        Ok(s) => s,
        Err(_) => {
            attach_selection_methods(lua, &selection_table)?;
            return Ok(selection_table);
        }
    };

    let mut count = 0;
    for el in html.select(&selector) {
        count += 1;
        let text = el.text().collect::<Vec<_>>().join(" ");
        let inner_html = el.inner_html();
        let attrs: Vec<(String, String)> = el
            .value()
            .attrs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let el_t = create_html_element(lua, text, inner_html, attrs)?;
        selection_table.set(count, el_t)?;
    }

    attach_selection_methods(lua, &selection_table)?;
    Ok(selection_table)
}

fn attach_selection_methods(lua: &Lua, selection_table: &Table) -> mlua::Result<()> {
    lua.load(
        r#"
        local t = ...
        function t:each(cb)
            for i = 1, #self do
                cb(i - 1, self[i])
            end
        end
        function t:first()
            return self[1] or create_dummy()
        end
        function t:last()
            return self[#self] or create_dummy()
        end
        "#,
    )
    .call::<()>(selection_table.clone())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_runtime_html_parsing() {
        let lua = Lua::new();
        setup_lua_env(&lua).unwrap();

        let script = r#"
            local Html = require("html")
            local doc = Html.parse("<div class='manga'><a href='/manga/123'>One Piece</a><span class='author'>Oda</span></div>")
            local link = doc:find(".manga a"):first()
            Name = link:text()
            Href = link:attr("href")

            Count = 0
            doc:find("a, span"):each(function(i, el)
                Count = Count + 1
            end)
        "#;

        lua.load(script).exec().unwrap();
        let name: String = lua.globals().get("Name").unwrap();
        let href: String = lua.globals().get("Href").unwrap();
        let count: i32 = lua.globals().get("Count").unwrap();

        assert_eq!(name, "One Piece");
        assert_eq!(href, "/manga/123");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_lua_scraper_search_manga() {
        let code = r#"
            function SearchManga(query)
                return {
                    { name = "Berserk", url = "https://example.com/berserk" },
                    { name = "Vagabond", url = "https://example.com/vagabond" }
                }
            end

            function MangaChapters(url)
                return {
                    { name = "Chapter 1", url = url .. "/1" }
                }
            end

            function ChapterPages(url)
                return {
                    { index = 0, url = url .. "/page1.jpg" }
                }
            end
        "#;

        let scraper = LuaScraper::from_string("TestSource", code);
        let mangas = scraper.search("test").unwrap();
        assert_eq!(mangas.len(), 2);
        assert_eq!(mangas[0].title, "Berserk");

        let chapters = scraper.chapters(&mangas[0].url).unwrap();
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Chapter 1");

        let pages = scraper.pages(&chapters[0].url).unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].url, "https://example.com/berserk/1/page1.jpg");
    }
}
