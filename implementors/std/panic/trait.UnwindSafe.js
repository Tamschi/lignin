(function() {var implementors = {};
implementors["lignin"] = [{"text":"impl&lt;R, T&gt; UnwindSafe for CallbackRegistration&lt;R, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S, T&gt; UnwindSafe for CallbackRef&lt;S, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for DomRef&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Comment","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Event","synthetic":true,"types":[]},{"text":"impl UnwindSafe for HtmlElement","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Text","synthetic":true,"types":[]},{"text":"impl&lt;'a, S&gt; UnwindSafe for Node&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: RefUnwindSafe + UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, S&gt; UnwindSafe for ReorderableFragment&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: RefUnwindSafe + UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, S&gt; UnwindSafe for Element&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: RefUnwindSafe + UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, S&gt; UnwindSafe for EventBinding&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for Attribute&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for ThreadBound","synthetic":true,"types":[]},{"text":"impl UnwindSafe for ThreadSafe","synthetic":true,"types":[]}];
implementors["lignin_html"] = [{"text":"impl&lt;'a, S&gt; UnwindSafe for Error&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()