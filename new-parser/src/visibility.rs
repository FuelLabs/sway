use crate::priv_prelude::*;

pub enum Visibility {
    Public {
        pub_token: PubToken,
    },
    Private {
        span: Span,
    },
}

impl Spanned for Visibility {
    fn span(&self) -> Span {
        match self {
            Visibility::Public { pub_token } => pub_token.span(),
            Visibility::Private { span } => span.clone(),
        }
    }
}

pub fn visibility() -> impl Parser<char, Visibility, Error = Cheap<char, Span>> + Clone {
    let public = {
        pub_token()
        .map(|pub_token| Visibility::Public { pub_token })
    };
    let private = {
        empty()
        .map_with_span(|(), span| Visibility::Private { span })
    };

    public
    .or(private)
}

