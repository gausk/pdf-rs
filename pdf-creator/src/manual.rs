use anyhow::Result;
use std::fs::File;
use std::io::{Seek, Write};

#[derive(Debug)]
pub struct PdfDocument {
    version: PdfVersion,
    catalog: Catalog,
    pages: Pages,
    page: Page,
    contents: ContentStream,
}

#[derive(Debug)]
pub enum PdfVersion {
    Pdf14,
}

impl PdfVersion {
    pub fn to_str(&self) -> &'static str {
        match self {
            PdfVersion::Pdf14 => "%PDF-1.4\n",
        }
    }
}

#[derive(Debug)]
pub struct Catalog {
    pages: ObjectRef,
}

impl Catalog {
    pub fn to_string(&self) -> String {
        format!(
            "1 0 obj\n<< /Type /Catalog /Pages {} {} R >>\nendobj\n",
            self.pages.id, self.pages.generation
        )
    }
}

#[derive(Debug)]
pub struct ObjectRef {
    id: u32,
    generation: u16,
}

#[derive(Debug)]
pub struct Pages {
    kids: Vec<ObjectRef>,
    count: usize,
}

impl Pages {
    pub fn to_string(&self) -> String {
        assert_eq!(self.kids.len(), self.count);
        assert_eq!(self.count, 1);
        format!(
            "2 0 obj\n<< /Type /Pages /Kids [{} {} R] /Count {} >>\nendobj\n",
            self.kids[0].id, self.kids[0].generation, self.count
        )
    }
}

#[derive(Debug)]
pub struct Page {
    parent: ObjectRef,
    media_box: [f32; 4],
    contents: ObjectRef,
    font_resources: FontResources,
}

impl Page {
    pub fn to_string(&self) -> String {
        format!(
            "3 0 obj\n<< /Type /Page /Parent {} {} R /MediaBox [{} {} {} {}] \
         /Contents {} {} R /Resources << /Font << /{} {} {} R >> >> >>\nendobj\n",
            self.parent.id,
            self.parent.generation,
            self.media_box[0],
            self.media_box[1],
            self.media_box[2],
            self.media_box[3],
            self.contents.id,
            self.contents.generation,
            self.font_resources.name,
            self.font_resources.object.id,
            self.font_resources.object.generation
        )
    }
}

#[derive(Debug)]
pub struct FontResources {
    name: String,
    object: ObjectRef,
}

#[derive(Debug)]
pub struct ContentStream {
    content: String,
}

impl ContentStream {
    pub fn to_string(&self) -> String {
        let stream = format!("BT\n/F1 24 Tf\n100 700 Td\n({}) Tj\nET\n", self.content);

        format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
            stream.len(),
            stream
        )
    }
}

impl PdfDocument {
    pub fn new(content: &str) -> Self {
        Self {
            version: PdfVersion::Pdf14,
            catalog: Catalog {
                pages: ObjectRef {
                    id: 2,
                    generation: 0,
                },
            },
            pages: Pages {
                kids: vec![ObjectRef {
                    id: 3,
                    generation: 0,
                }],
                count: 1,
            },
            page: Page {
                parent: ObjectRef {
                    id: 2,
                    generation: 0,
                },
                media_box: [0.0, 0.0, 595.0, 842.0],
                contents: ObjectRef {
                    id: 4,
                    generation: 0,
                },
                font_resources: FontResources {
                    name: String::from("F1"),
                    object: ObjectRef {
                        id: 5,
                        generation: 0,
                    },
                },
            },
            contents: ContentStream {
                content: content.to_string(),
            },
        }
    }

    pub fn create(&self) -> Result<()> {
        let mut file = File::create("./manual.pdf")?;
        file.write_all(self.version.to_str().as_bytes())?;

        let mut offsets: Vec<u64> = Vec::new();

        macro_rules! write_pdf {
            ($s:expr) => {{
                offsets.push(file.stream_position()?);
                file.write_all($s.as_bytes())?;
            }};
        }

        write_pdf!(self.catalog.to_string());
        write_pdf!(self.pages.to_string());
        write_pdf!(self.page.to_string());
        write_pdf!(self.contents.to_string());

        write_pdf!("5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

        let xref_pos = file.stream_position()?;
        file.write_all(b"xref\n0 6\n0000000000 65535 f\n")?;

        for off in offsets {
            file.write_all(format!("{:010} 00000 n\n", off).as_bytes())?;
        }

        file.write_all(
            format!(
                "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
                xref_pos
            )
            .as_bytes(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_create_pdf() {
        let doc = PdfDocument::new("Hello Gaurav!. Great work");
        doc.create().unwrap();
    }
}
