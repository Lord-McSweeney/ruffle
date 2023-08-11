package flash.net {
    import flash.events.EventDispatcher;
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    // NOTE: this entire class is a stub.
    // Thankfully (hopefully) a lot of code like Mochicrypt doesn't actually require this to... well do anything.
    public class LocalConnection extends EventDispatcher {
        internal var _connectionIndex:int = -1;
        internal static var _connections:Array = [];

        public var client: Object;

        public function LocalConnection() {
            this.client = this;
        }

        public native function get domain():String;

        public native function close():void;

        public native function connect(connectionName:String):void;

        public native function send(connectionName: String, methodName: String, ... arguments):void;

        public function allowDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowDomain");
        }

        public function allowInsecureDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowInsecureDomain");
        }
    }
}
