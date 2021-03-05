var searchIndex = JSON.parse('{\
"lignin":{"doc":"<code>lignin</code>, named after the structural polymer found in …","i":[[0,"auto_safety","lignin","Transitive (across function boundaries) [<code>ThreadSafety</code>] …",null,null],[8,"AutoSafe","lignin::auto_safety","Deanonymize towards the general ([<code>ThreadBound</code>]) case. …",null,null],[11,"deanonymize","","Deanonymize towards a compatible concrete type.",0,[[]]],[8,"Deanonymize","","Deanonymize towards the special ([<code>ThreadSafe</code>]) case. <strong>This …",null,null],[11,"deanonymize","","Deanonymize towards a compatible concrete type.",1,[[]]],[8,"Align","","Contextually thread-binds an instance, or not. Use only …",null,null],[11,"align","","Contextually thread-binds an instance, or not. Use only …",2,[[]]],[11,"align_ref","","Contextually thread-binds a reference, or not. Use only …",2,[[]]],[14,"AutoSafe_alias","","Mainly for use by frameworks. Canonically located at …",null,null],[0,"callback_registry","lignin","Callback registry plumbing, for renderers and app runners …",null,null],[3,"CallbackRegistration","lignin::callback_registry","A callback registration handle that should be held onto …",null,null],[11,"new","","Creates a new [<code>CallbackRegistration<R, T></code>] with the given …",3,[[["pin",3]]]],[11,"to_ref_thread_bound","","Creates a [<code>ThreadBound</code>] [<code>CallbackRef</code>] from this […",3,[[],[["callbackref",3],["threadbound",3]]]],[11,"to_ref","","Creates a [<code>ThreadSafe</code>] [<code>CallbackRef</code>] from this […",3,[[],[["threadsafe",3],["callbackref",3]]]],[8,"ToRefThreadBoundFallback","","Provides a fallback alternative implementation to […",null,null],[10,"to_ref","","See [<code>CallbackRegistration::to_ref</code>], except that this …",4,[[],[["callbackref",3],["threadbound",3]]]],[3,"CallbackRef","","<code>Vdom</code> A callback reference linked to a […",null,null],[11,"call","","Invokes the stored handler with the stored receiver and …",5,[[]]],[5,"registry_exhaustion","","Indicates how exhausted the global callback registry is …",null,[[]]],[5,"reset_callback_registry","","Tries to rewind the total callback registration counter …",null,[[],["result",4]]],[5,"yet_more_unsafe_force_clear_callback_registry","","Clears the callback registry entirely and resets the …",null,[[]]],[0,"web","lignin","Erasable web type stand-ins used as callback parameters.",null,null],[4,"DomRef","lignin::web","Used as DOM reference callback parameter. (Expand for …",null,null],[13,"Added","","When constructing the DOM, this variant is passed <strong>after</strong> …",6,null],[13,"Removing","","When tearing down the DOM, this variant is passed <strong>before</strong> …",6,null],[3,"Comment","","Erasable stand-in for <code>web_sys::Comment</code> used as callback …",null,null],[11,"new","","Creates a new [`",7,[[["comment",3]]]],[3,"Event","","Erasable stand-in for <code>web_sys::Event</code> used as callback …",null,null],[11,"new","","Creates a new [`",8,[[["event",3]]]],[3,"HtmlElement","","Erasable stand-in for <code>web_sys::HtmlElement</code> used as …",null,null],[11,"new","","Creates a new [`",9,[[["htmlelement",3]]]],[3,"Text","","Erasable stand-in for <code>web_sys::Text</code> used as callback …",null,null],[11,"new","","Creates a new [`",10,[[["text",3]]]],[8,"Materialize","","Convert a DOM stand-in to its web type value. This is a …",null,null],[10,"materialize","","Convert a DOM stand-in to its web type value. This is a …",11,[[]]],[4,"Node","lignin","[<code>Vdom</code>] A single generic VDOM node.",null,null],[13,"Comment","","Represents a <em><strong>Comment</strong></em> node.",12,null],[12,"comment","lignin::Node","The comment\'s body, as unescaped plaintext.",13,null],[12,"dom_binding","","Registers for <em><strong>Comment</strong></em> reference updates.",13,null],[13,"Element","lignin","Represents a single <em><strong>HTMLElement</strong></em>.",12,null],[12,"element","lignin::Node","The [<code>Element</code>] to render.",14,null],[12,"dom_binding","","Registers for <em><strong>HTMLElement</strong></em> reference updates.",14,null],[13,"Memoized","lignin","DOM-transparent. This variant uses shallow comparison and …",12,null],[12,"state_key","lignin::Node","A value that\'s (very likely to be) distinct between VDOM …",15,null],[12,"content","","The VDOM tree memoized by this [<code>Node</code>].",15,null],[13,"Multi","lignin","DOM-transparent. Represents a sequence of VDOM nodes.",12,null],[13,"Keyed","","A sequence of VDOM nodes that\'s transparent at rest, but …",12,null],[13,"Text","","Represents a <em><strong>Text</strong></em> node.",12,null],[12,"text","lignin::Node","The <code>Text</code>\'s <em><strong>Node.textContent</strong></em>.",16,null],[12,"dom_binding","","Registers for <em><strong>Text</strong></em> reference updates.",16,null],[13,"RemnantSite","lignin","Currently unused.",12,null],[3,"ReorderableFragment","","[<code>Vdom</code>] A VDOM node that has its DOM identity preserved …",null,null],[12,"dom_key","","A key uniquely identifying a [<code>ReorderableFragment</code>] within …",17,null],[12,"content","","The [<code>Node</code>] to render from this [<code>ReorderableFragment</code>].",17,null],[3,"Element","","[<code>Vdom</code>] Represents a single <em><strong>HTMLElement</strong></em> as <code>name</code>, <code>attributes</code>…",null,null],[12,"name","","The <em><strong>Element.tag_name</strong></em>.",18,null],[12,"attributes","","The <em><strong>Element.attributes</strong></em>.",18,null],[12,"content","","Maps to <em><strong>Node.childNodes</strong></em>.",18,null],[12,"event_bindings","","DOM event bindings requested by a component.",18,null],[3,"EventBinding","","[<code>Vdom</code>] Represents a single DOM event binding with <code>name</code> …",null,null],[12,"name","","The event name.",19,null],[12,"callback","","A callback reference created via [<code>CallbackRegistration</code>].",19,null],[3,"Attribute","","[<code>Vdom</code>] Represents a single HTML <em><strong>Attr</strong></em> with <code>name</code> and <code>value</code>.",null,null],[12,"name","","The <em><strong>name</strong></em>.",20,null],[12,"value","","The unescaped <em><strong>value</strong></em>.",20,null],[8,"ThreadSafety","","Marker trait for thread-safety tokens.",null,null],[3,"ThreadBound","","[<code>ThreadSafety</code>] marker for <code>!Send + !Sync</code>.",null,null],[12,"0","","Neither [<code>Send</code>] nor [<code>Sync</code>].",21,null],[12,"1","","Uninhabited.",21,null],[3,"ThreadSafe","","[<code>ThreadSafety</code>] marker for <code>Send + Sync</code>.",null,null],[12,"0","","This field here technically doesn\'t matter, but I went …",22,null],[12,"1","","Uninhabited.",22,null],[8,"Vdom","","Marker trait for VDOM data types, which (almost) all vary …",null,null],[16,"ThreadSafety","","The [<code>ThreadSafety</code>] of the [<code>Vdom</code>] type, either [<code>ThreadSafe</code>]…",23,null],[14,"AutoSafe_alias","","Mainly for use by frameworks. Canonically located at …",null,null],[11,"from","lignin::callback_registry","",3,[[]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"into","","",3,[[]]],[11,"try_into","","",3,[[],["result",4]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"from","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"into","","",5,[[]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"to_owned","","",5,[[]]],[11,"clone_into","","",5,[[]]],[11,"from","lignin::web","",6,[[]]],[11,"borrow","","",6,[[]]],[11,"borrow_mut","","",6,[[]]],[11,"try_from","","",6,[[],["result",4]]],[11,"into","","",6,[[]]],[11,"try_into","","",6,[[],["result",4]]],[11,"type_id","","",6,[[],["typeid",3]]],[11,"to_owned","","",6,[[]]],[11,"clone_into","","",6,[[]]],[11,"from","","",7,[[]]],[11,"borrow","","",7,[[]]],[11,"borrow_mut","","",7,[[]]],[11,"try_from","","",7,[[],["result",4]]],[11,"into","","",7,[[]]],[11,"try_into","","",7,[[],["result",4]]],[11,"type_id","","",7,[[],["typeid",3]]],[11,"to_owned","","",7,[[]]],[11,"clone_into","","",7,[[]]],[11,"from","","",8,[[]]],[11,"borrow","","",8,[[]]],[11,"borrow_mut","","",8,[[]]],[11,"try_from","","",8,[[],["result",4]]],[11,"into","","",8,[[]]],[11,"try_into","","",8,[[],["result",4]]],[11,"type_id","","",8,[[],["typeid",3]]],[11,"to_owned","","",8,[[]]],[11,"clone_into","","",8,[[]]],[11,"from","","",9,[[]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"into","","",9,[[]]],[11,"try_into","","",9,[[],["result",4]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"to_owned","","",9,[[]]],[11,"clone_into","","",9,[[]]],[11,"from","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"into","","",10,[[]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"to_owned","","",10,[[]]],[11,"clone_into","","",10,[[]]],[11,"from","lignin","",12,[[]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"into","","",12,[[]]],[11,"try_into","","",12,[[],["result",4]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"to_owned","","",12,[[]]],[11,"clone_into","","",12,[[]]],[11,"from","","",17,[[]]],[11,"borrow","","",17,[[]]],[11,"borrow_mut","","",17,[[]]],[11,"try_from","","",17,[[],["result",4]]],[11,"into","","",17,[[]]],[11,"try_into","","",17,[[],["result",4]]],[11,"type_id","","",17,[[],["typeid",3]]],[11,"to_owned","","",17,[[]]],[11,"clone_into","","",17,[[]]],[11,"from","","",18,[[]]],[11,"borrow","","",18,[[]]],[11,"borrow_mut","","",18,[[]]],[11,"try_from","","",18,[[],["result",4]]],[11,"into","","",18,[[]]],[11,"try_into","","",18,[[],["result",4]]],[11,"type_id","","",18,[[],["typeid",3]]],[11,"to_owned","","",18,[[]]],[11,"clone_into","","",18,[[]]],[11,"from","","",19,[[]]],[11,"borrow","","",19,[[]]],[11,"borrow_mut","","",19,[[]]],[11,"try_from","","",19,[[],["result",4]]],[11,"into","","",19,[[]]],[11,"try_into","","",19,[[],["result",4]]],[11,"type_id","","",19,[[],["typeid",3]]],[11,"to_owned","","",19,[[]]],[11,"clone_into","","",19,[[]]],[11,"from","","",20,[[]]],[11,"borrow","","",20,[[]]],[11,"borrow_mut","","",20,[[]]],[11,"try_from","","",20,[[],["result",4]]],[11,"into","","",20,[[]]],[11,"try_into","","",20,[[],["result",4]]],[11,"type_id","","",20,[[],["typeid",3]]],[11,"to_owned","","",20,[[]]],[11,"clone_into","","",20,[[]]],[11,"from","","",21,[[]]],[11,"borrow","","",21,[[]]],[11,"borrow_mut","","",21,[[]]],[11,"try_from","","",21,[[],["result",4]]],[11,"into","","",21,[[]]],[11,"try_into","","",21,[[],["result",4]]],[11,"type_id","","",21,[[],["typeid",3]]],[11,"to_owned","","",21,[[]]],[11,"clone_into","","",21,[[]]],[11,"from","","",22,[[]]],[11,"borrow","","",22,[[]]],[11,"borrow_mut","","",22,[[]]],[11,"try_from","","",22,[[],["result",4]]],[11,"into","","",22,[[]]],[11,"try_into","","",22,[[],["result",4]]],[11,"type_id","","",22,[[],["typeid",3]]],[11,"to_owned","","",22,[[]]],[11,"clone_into","","",22,[[]]],[11,"to_ref","lignin::callback_registry","",3,[[],[["callbackref",3],["threadbound",3]]]],[11,"materialize","lignin::web","",6,[[],["domref",4]]],[11,"materialize","","",7,[[],["comment",3]]],[11,"materialize","","",8,[[],["event",3]]],[11,"materialize","","",9,[[],["htmlelement",3]]],[11,"materialize","","",10,[[],["text",3]]],[11,"from","lignin","Unreachable. Available as compatibility marker when …",21,[[["threadsafe",3]]]],[11,"from","lignin::callback_registry","",5,[[["callbackref",3],["threadsafe",3]]]],[11,"from","","",5,[[["callbackregistration",3]]]],[11,"from","","",5,[[["callbackregistration",3]]]],[11,"from","lignin","",18,[[["element",3],["threadsafe",3]]]],[11,"from","","",19,[[["eventbinding",3],["threadsafe",3]]]],[11,"from","","",12,[[["node",4],["threadsafe",3]]]],[11,"from","","",17,[[["reorderablefragment",3],["threadsafe",3]]]],[11,"from","","",12,[[["element",3]]]],[11,"from","","",12,[[["element",3]]]],[11,"from","","",12,[[]]],[11,"from","","",12,[[]]],[11,"from","","",12,[[]]],[11,"from","","",12,[[]]],[11,"fmt","lignin::callback_registry","",3,[[["formatter",3]],["result",6]]],[11,"fmt","lignin::web","",6,[[["formatter",3]],["result",6]]],[11,"fmt","","",7,[[["formatter",3]],["result",6]]],[11,"fmt","","",8,[[["formatter",3]],["result",6]]],[11,"fmt","","",9,[[["formatter",3]],["result",6]]],[11,"fmt","","",10,[[["formatter",3]],["result",6]]],[11,"fmt","lignin::callback_registry","",5,[[["formatter",3]],["result",6]]],[11,"fmt","lignin","",18,[[["formatter",3]],["result",6]]],[11,"fmt","","",19,[[["formatter",3]],["result",6]]],[11,"fmt","","",12,[[["formatter",3]],["result",6]]],[11,"fmt","","",17,[[["formatter",3]],["result",6]]],[11,"fmt","","",20,[[["formatter",3]],["result",6]]],[11,"fmt","","",21,[[["formatter",3]],["result",6]]],[11,"fmt","","",22,[[["formatter",3]],["result",6]]],[11,"eq","lignin::web","",6,[[["domref",4]]]],[11,"ne","","",6,[[["domref",4]]]],[11,"eq","lignin::callback_registry","",5,[[["callbackref",3]]]],[11,"eq","lignin","",18,[[["element",3]]]],[11,"eq","","",19,[[["eventbinding",3]]]],[11,"eq","","",12,[[["node",4]]]],[11,"eq","","",17,[[["reorderablefragment",3]]]],[11,"eq","","",20,[[["attribute",3]]]],[11,"ne","","",20,[[["attribute",3]]]],[11,"eq","","",21,[[["threadbound",3]]]],[11,"ne","","",21,[[["threadbound",3]]]],[11,"eq","","",22,[[["threadsafe",3]]]],[11,"ne","","",22,[[["threadsafe",3]]]],[11,"cmp","lignin::web","",6,[[["domref",4]],["ordering",4]]],[11,"cmp","lignin::callback_registry","",5,[[],["ordering",4]]],[11,"cmp","lignin","",18,[[],["ordering",4]]],[11,"cmp","","",19,[[],["ordering",4]]],[11,"cmp","","",12,[[],["ordering",4]]],[11,"cmp","","",17,[[],["ordering",4]]],[11,"cmp","","",20,[[["attribute",3]],["ordering",4]]],[11,"cmp","","",21,[[["threadbound",3]],["ordering",4]]],[11,"cmp","","",22,[[["threadsafe",3]],["ordering",4]]],[11,"partial_cmp","lignin::web","",6,[[["domref",4]],[["option",4],["ordering",4]]]],[11,"lt","","",6,[[["domref",4]]]],[11,"le","","",6,[[["domref",4]]]],[11,"gt","","",6,[[["domref",4]]]],[11,"ge","","",6,[[["domref",4]]]],[11,"partial_cmp","lignin::callback_registry","",5,[[["callbackref",3]],[["ordering",4],["option",4]]]],[11,"partial_cmp","lignin","",18,[[["element",3]],[["ordering",4],["option",4]]]],[11,"partial_cmp","","",19,[[["eventbinding",3]],[["ordering",4],["option",4]]]],[11,"partial_cmp","","",12,[[["node",4]],[["ordering",4],["option",4]]]],[11,"partial_cmp","","",17,[[["reorderablefragment",3]],[["ordering",4],["option",4]]]],[11,"partial_cmp","","",20,[[["attribute",3]],[["option",4],["ordering",4]]]],[11,"lt","","",20,[[["attribute",3]]]],[11,"le","","",20,[[["attribute",3]]]],[11,"gt","","",20,[[["attribute",3]]]],[11,"ge","","",20,[[["attribute",3]]]],[11,"partial_cmp","","",21,[[["threadbound",3]],[["option",4],["ordering",4]]]],[11,"lt","","",21,[[["threadbound",3]]]],[11,"le","","",21,[[["threadbound",3]]]],[11,"gt","","",21,[[["threadbound",3]]]],[11,"ge","","",21,[[["threadbound",3]]]],[11,"partial_cmp","","",22,[[["threadsafe",3]],[["option",4],["ordering",4]]]],[11,"lt","","",22,[[["threadsafe",3]]]],[11,"le","","",22,[[["threadsafe",3]]]],[11,"gt","","",22,[[["threadsafe",3]]]],[11,"ge","","",22,[[["threadsafe",3]]]],[11,"drop","lignin::callback_registry","",3,[[]]],[11,"hash","lignin::web","",6,[[]]],[11,"hash","lignin::callback_registry","",5,[[]]],[11,"hash","lignin","",18,[[]]],[11,"hash","","",19,[[]]],[11,"hash","","",12,[[]]],[11,"hash","","",17,[[]]],[11,"hash","","",20,[[]]],[11,"hash","","",21,[[]]],[11,"hash","","",22,[[]]],[11,"clone","lignin::web","",6,[[],["domref",4]]],[11,"clone","","",7,[[],["comment",3]]],[11,"clone","","",8,[[],["event",3]]],[11,"clone","","",9,[[],["htmlelement",3]]],[11,"clone","","",10,[[],["text",3]]],[11,"clone","lignin::callback_registry","",5,[[]]],[11,"clone","lignin","",18,[[]]],[11,"clone","","",19,[[]]],[11,"clone","","",12,[[]]],[11,"clone","","",17,[[]]],[11,"clone","","",20,[[],["attribute",3]]],[11,"clone","","",21,[[],["threadbound",3]]],[11,"clone","","",22,[[],["threadsafe",3]]],[11,"deanonymize","","When called on an opaque type, deanonymizes it into the …",20,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",20,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",20,[[]]],[11,"deanonymize","","When called on an opaque type, deanonymizes it into the …",18,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",18,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",18,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",18,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",18,[[]]],[11,"deanonymize","","When called on an opaque type, deanonymizes it into the …",19,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",19,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",19,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",19,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",19,[[]]],[11,"deanonymize","","When called on an opaque type, deanonymizes it into the …",12,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",12,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",12,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",12,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",12,[[]]],[11,"deanonymize","","When called on an opaque type, deanonymizes it into the …",17,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",17,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",17,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",17,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",17,[[]]],[11,"deanonymize","lignin::callback_registry","When called on an opaque type, deanonymizes it into the …",5,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",5,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",5,[[]]],[11,"prefer_thread_safe","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",5,[[]]],[11,"prefer_thread_safe_ref","","Gently nudges the compiler to choose the [<code>ThreadSafe</code>] …",5,[[]]],[11,"dom_len","lignin","Calculates the aggregate surface level length of this […",12,[[]]],[11,"dom_empty","","Determines whether this [<code>Node</code>] represents no <em><strong>Node</strong></em>s at all.",12,[[]]]],"p":[[8,"AutoSafe"],[8,"Deanonymize"],[8,"Align"],[3,"CallbackRegistration"],[8,"ToRefThreadBoundFallback"],[3,"CallbackRef"],[4,"DomRef"],[3,"Comment"],[3,"Event"],[3,"HtmlElement"],[3,"Text"],[8,"Materialize"],[4,"Node"],[13,"Comment"],[13,"Element"],[13,"Memoized"],[13,"Text"],[3,"ReorderableFragment"],[3,"Element"],[3,"EventBinding"],[3,"Attribute"],[3,"ThreadBound"],[3,"ThreadSafe"],[8,"Vdom"]]},\
"lignin_html":{"doc":"An HTML renderer for <code>lignin</code> that does <em>some</em> syntactic and <em>…","i":[[5,"render_document","lignin_html","Renders <code>vdom</code> into <code>target</code> as HTML document <em>with</em> <em><strong>DOCTYPE</strong></em>.",null,[[["node",4]],[["result",4],["error",3]]]],[5,"render_fragment","","Renders <code>vdom</code> into <code>target</code> as HTML fragment <em>without</em> <em><strong>DOCTYPE</strong></em>.",null,[[["node",4]],[["result",4],["error",3]]]],[3,"Error","","",null,null],[11,"from","","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"into","","",0,[[]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"to_string","","",0,[[],["string",3]]],[11,"from","","",0,[[["error",3]]]],[11,"fmt","","",0,[[["formatter",3]],["result",6]]],[11,"fmt","","",0,[[["formatter",3]],["result",6]]],[11,"source","","",0,[[],[["error",8],["option",4]]]]],"p":[[3,"Error"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);