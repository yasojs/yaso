use rquickjs::class::{JsClass, OwnedBorrow, Trace, Tracer};
use rquickjs::function::{Func, Rest, This};
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{
    Class, Ctx, Exception, Function, Object, Result as QuickJsResult, String as JsString, Symbol,
    Value,
};

use std::sync::{Arc, RwLock};

#[derive(Clone, PartialEq)]
pub enum EventKey<'js> {
    Symbol(Symbol<'js>),
    String(String),
}

impl<'js> EventKey<'js> {
    pub fn from_value(ctx: &Ctx<'js>, value: Value<'js>) -> QuickJsResult<EventKey<'js>> {
        if value.is_string() {
            Ok(EventKey::String(value.get()?))
        } else if value.is_symbol() {
            Ok(EventKey::Symbol(value.get()?))
        } else {
            Err(Exception::throw_message(ctx, "Expected a string or symbol"))
        }
    }
}

pub struct EventItem<'js> {
    callback: Function<'js>,
    once: bool,
}

pub type Events<'js> = Arc<RwLock<Vec<(EventKey<'js>, Vec<EventItem<'js>>)>>>;

#[rquickjs::class]
pub struct EventEmitter<'js> {
    pub events: Events<'js>,
}

impl<'js> Trace<'js> for EventEmitter<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.trace_event_emitter(tracer);
    }
}

impl<'js> EventEmitterTrait<'js> for EventEmitter<'js> {
    fn get_events(&self) -> Events<'js> {
        self.events.clone()
    }
}

#[rquickjs::methods]
impl<'js> EventEmitter<'js> {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            #[allow(clippy::arc_with_non_send_sync)]
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub trait EventEmitterTrait<'js>: JsClass<'js> + 'js + Sized {
    fn get_events(&self) -> Events<'js>;

    fn get_event_names(
        this: This<OwnedBorrow<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> QuickJsResult<Vec<Value<'js>>> {
        let events = this.get_events();
        let events = events.read().unwrap();

        let mut names = Vec::with_capacity(events.len());

        for (key, _) in events.iter() {
            let name = match key {
                EventKey::Symbol(symbol) => symbol.clone().into(),
                EventKey::String(string) => JsString::from_str(ctx.clone(), string)?.into(),
            };

            names.push(name)
        }

        Ok(names)
    }

    fn trace_event_emitter<'a>(&self, tracer: Tracer<'a, 'js>) {
        let events = self.get_events();
        let events = events.read().unwrap();

        for (key, items) in events.iter() {
            if let EventKey::Symbol(symbol) = key {
                tracer.mark(symbol);
            }

            for item in items {
                tracer.mark(&item.callback);
            }
        }
    }

    fn add_event_emitter_prototype(ctx: &Ctx<'js>) -> QuickJsResult<Object<'js>> {
        let prototype = Class::<Self>::prototype(ctx.clone()).ok_or(Exception::throw_message(
            ctx,
            "EventEmitter Prototype not found",
        ))?;

        prototype.set("on", Func::from(Self::on))?;
        prototype.set("addListener", Func::from(Self::on))?;

        prototype.set("off", Func::from(Self::remove_event_listener))?;
        prototype.set("removeListener", Func::from(Self::remove_event_listener))?;

        prototype.set("once", Func::from(Self::once))?;

        prototype.set("prependListener", Func::from(Self::prepend_listener))?;
        prototype.set(
            "prependOnceListener",
            Func::from(Self::prepend_once_listener),
        )?;

        prototype.set("emit", Func::from(Self::emit))?;

        prototype.set("eventNames", Func::from(Self::get_event_names))?;

        Ok(prototype)
    }

    fn remove_event_listener(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        event_callback: Function<'js>,
    ) -> QuickJsResult<Class<'js, Self>> {
        let events = this.borrow().get_events();
        let mut events = events.write().unwrap();

        let event_key = EventKey::from_value(&ctx, event_key)?;

        if let Some(event_index) = events.iter().position(|(k, _)| k == &event_key) {
            let event_items = &mut events.get_mut(event_index).unwrap().1;

            if let Some(event_item_index) = event_items
                .iter()
                .position(|i| i.callback == event_callback)
            {
                event_items.remove(event_item_index);

                if event_items.is_empty() {
                    events.remove(event_index);
                }
            }
        }

        Ok(this.0)
    }

    fn add_event_listener(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        event_callback: Function<'js>,
        prepend: bool,
        once: bool,
    ) -> QuickJsResult<Class<'js, Self>> {
        let events = this.borrow().get_events();
        let mut events = events.write().unwrap();

        let event_key = EventKey::from_value(&ctx, event_key)?;

        let event_item = EventItem {
            callback: event_callback,
            once,
        };

        let event_items = match events.iter_mut().find(|(k, _)| k == &event_key) {
            Some((_, entry_items)) => entry_items,
            None => {
                events.push((event_key, Vec::new()));

                &mut events.last_mut().unwrap().1
            }
        };

        if prepend {
            event_items.insert(0, event_item);
        } else {
            event_items.push(event_item);
        }

        Ok(this.0)
    }

    fn on(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        event_callback: Function<'js>,
    ) -> QuickJsResult<Class<'js, Self>> {
        Self::add_event_listener(this, ctx, event_key, event_callback, false, false)
    }

    fn once(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        event_callback: Function<'js>,
    ) -> QuickJsResult<Class<'js, Self>> {
        Self::add_event_listener(this, ctx, event_key, event_callback, false, true)
    }

    fn prepend_listener(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        event_callback: Function<'js>,
    ) -> QuickJsResult<Class<'js, Self>> {
        Self::add_event_listener(this, ctx, event_key, event_callback, true, false)
    }

    fn prepend_once_listener(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        event_callback: Function<'js>,
    ) -> QuickJsResult<Class<'js, Self>> {
        Self::add_event_listener(this, ctx, event_key, event_callback, true, true)
    }

    fn emit(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_key: Value<'js>,
        args: Rest<Value<'js>>,
    ) -> QuickJsResult<()> {
        let events = this.borrow().get_events();
        let mut events = events.write().unwrap();

        let event_key = EventKey::from_value(&ctx, event_key)?;

        if let Some(event_index) = events.iter().position(|(k, _)| k == &event_key) {
            let event_items = &mut events.get_mut(event_index).unwrap().1;

            let mut callbacks = Vec::with_capacity(event_items.len());

            event_items.retain(|event_item: &EventItem<'_>| {
                callbacks.push(event_item.callback.clone());

                !event_item.once
            });

            if event_items.is_empty() {
                events.remove(event_index);
            }

            for callback in callbacks {
                let args = args.iter().map(|arg| arg.to_owned()).collect();
                let args = Rest(args);
                let this = This(this.clone());

                callback.call((this, args))?;
            }
        }

        Ok(())
    }
}

pub struct EventsModule;

impl ModuleDef for EventsModule {
    fn declare(declare: &mut Declarations) -> QuickJsResult<()> {
        declare.declare(stringify!(EventEmitter))?;
        declare.declare("default")?;

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &mut Exports<'js>) -> QuickJsResult<()> {
        let event_emitter_constructor = Class::<EventEmitter>::create_constructor(ctx)?
            .expect("Could not create EventEmitter constructor");

        event_emitter_constructor
            .set(stringify!(EventEmitter), event_emitter_constructor.clone())?;

        exports.export(stringify!(EventEmitter), event_emitter_constructor.clone())?;

        exports.export("default", event_emitter_constructor)?;

        EventEmitter::add_event_emitter_prototype(ctx)?;

        Ok(())
    }
}
