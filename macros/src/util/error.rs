/// Used to create a error enum that converts into a [`Diagnostic`].
///
/// [`Diagnostic`]: proc_macro_error::Diagnostic
macro_rules! diagnostic_error_enum {
    ($(#[$meta: meta])* $visibility: vis enum $name: ident {
        $(
            #[error($($error: tt)*), span = $error_span: expr]
            $(#[note($($note: tt)*)$(, span = $note_span: expr)?])*
            $(#[help($($help: tt)*)$(, span = $help_span: expr)?])*
            $(#[err($($err: tt)*)$(, span = $err_span: expr)?])*
            $(#[source($source: ident)])?
            $variant: ident {
                $($variant_field: ident: $variant_field_type: ty),*
            },
        )*
    }) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $visibility enum $name
        {
            $(
                $variant {
                    $($variant_field: $variant_field_type),*
                },
            )*
        }

        impl From<$name> for ::proc_macro_error::Diagnostic
        {
            #[must_use]
            fn from(err: $name) -> Self
            {
                let (error, span, notes, helps, errs, source): (
                    String,
                    ::proc_macro2::Span,
                    Vec<(String, ::proc_macro2::Span)>,
                    Vec<(String, ::proc_macro2::Span)>,
                    Vec<(String, ::proc_macro2::Span)>,
                    Option<::proc_macro_error::Diagnostic>
                ) = match err {
                    $(
                        $name::$variant {
                            $($variant_field),*
                        } => {
                            (
                                format!($($error)*),
                                $error_span,
                                vec![$(
                                    (
                                        format!($($note)*),
                                        $crate::util::or!(
                                            ($($note_span)?)
                                            else (::proc_macro2::Span::call_site())
                                        )
                                    )
                                ),*],
                                vec![$(
                                    (
                                        format!($($help)*),
                                        $crate::util::or!(
                                            ($($help_span)?)
                                            else (::proc_macro2::Span::call_site())
                                        )
                                    )
                                ),*],
                                vec![$(
                                    (
                                        format!($($err)*),
                                        $crate::util::or!(
                                            ($($err_span)?)
                                            else (::proc_macro2::Span::call_site())
                                        )
                                    )
                                ),*],
                                $crate::util::to_option!($($source.into())?)
                            )
                        }
                    ),*
                };

                if let Some(source_diagnostic) = source {
                    source_diagnostic.emit();
                }

                let mut diagnostic = ::proc_macro_error::Diagnostic::spanned(
                    span,
                    ::proc_macro_error::Level::Error,
                    error
                );

                if !notes.is_empty() {
                    for (note, note_span) in notes {
                        diagnostic = diagnostic.span_note(note_span, note);
                    }
                }

                if !helps.is_empty() {
                    for (help, help_span) in helps {
                        diagnostic = diagnostic.span_help(help_span, help);
                    }
                }

                if !errs.is_empty() {
                    for (err, err_span) in errs {
                        diagnostic = diagnostic.span_error(err_span, err);
                    }
                }

                diagnostic
            }
        }

    };
}

pub(crate) use diagnostic_error_enum;
