(function() {var implementors = {};
implementors["lignin"] = [{"text":"impl&lt;R:&nbsp;Debug, C:&nbsp;Debug&gt; Debug for CallbackRegistration&lt;R, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: CallbackSignature,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug&gt; Debug for DomRef&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl Debug for Comment","synthetic":false,"types":[]},{"text":"impl Debug for Event","synthetic":false,"types":[]},{"text":"impl Debug for HtmlElement","synthetic":false,"types":[]},{"text":"impl Debug for SvgElement","synthetic":false,"types":[]},{"text":"impl Debug for Text","synthetic":false,"types":[]},{"text":"impl&lt;S, C&gt; Debug for CallbackRef&lt;S, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: CallbackSignature,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S&gt; Debug for Element&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S&gt; Debug for EventBinding&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S&gt; Debug for Node&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S&gt; Debug for ReorderableFragment&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Debug for Attribute&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl Debug for ThreadBound","synthetic":false,"types":[]},{"text":"impl Debug for ThreadSafe","synthetic":false,"types":[]}];
implementors["lignin_html"] = [{"text":"impl&lt;'a, S:&nbsp;Debug + ThreadSafety&gt; Debug for Error&lt;'a, S&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()