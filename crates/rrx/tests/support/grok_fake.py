#!/usr/bin/env python3
import json,os,sys,uuid,sqlite3,time,pathlib
synthetic=json.loads(pathlib.Path(sys.argv[0]).with_suffix('.json').read_text())
if synthetic.get('RRX_PYTHON_OBSERVED'):pathlib.Path(synthetic['RRX_PYTHON_OBSERVED']).write_text(json.dumps({'executable':sys.executable,'version_info':list(sys.version_info)}))
mode=synthetic.get('RRX_MODE','good');sid=None;prompt=None;calls=0;model='native-default';effort='high'
if synthetic.get('RRX_ENVIRONMENT_OBSERVED'):
 expected=json.loads(synthetic['RRX_ENVIRONMENT_EXPECTED'])
 matches={key:os.getenv(key)==value for key,value in expected.items()}
 assert all(matches.values()),'synthetic environment canary mismatch'
 pathlib.Path(synthetic['RRX_ENVIRONMENT_OBSERVED']).write_text(json.dumps(matches))
if synthetic.get('RRX_SPAWN_OBSERVED'):pathlib.Path(synthetic['RRX_SPAWN_OBSERVED']).write_text('native process started')
profile=pathlib.Path(sys.argv[sys.argv.index('--agent-profile')+1]).read_text();decision='name: rururunx-decision' in profile
assert 'injectDefaultTools: false' in profile and 'GrokBuild:read_file' in profile and 'GrokBuild:search_replace' in profile
assert 'web_search, x_search, web_fetch' in profile and '--disable-web-search' in sys.argv
assert ('--sandbox' in sys.argv) and sys.argv[sys.argv.index('--sandbox')+1]==('read-only' if decision else 'strict')
def send(v):print(json.dumps(v),flush=True)
def update(v):send({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':sid,'update':v,'_meta':{'promptId':prompt}}})
def fs(method,path,content=None):
 p={'sessionId':sid,'path':path};p.update({'content':content} if content is not None else {})
 send({'jsonrpc':'2.0','id':'callback','method':method,'params':p});return json.loads(sys.stdin.readline())
def tool(name,path,n):
 global calls
 calls+=1;key='target_file' if name=='read_file' else 'file_path'
 update({'sessionUpdate':'tool_call','toolCallId':str(n),'rawInput':{key:path},'_meta':{'x.ai/tool':{'name':name}}})
 return str(n)
def done(n,failed=False):update({'sessionUpdate':'tool_call_update','toolCallId':str(n),'status':'failed' if failed else 'completed'})
for line in sys.stdin:
 d=json.loads(line);method=d.get('method');p=d.get('params',{});result={}
 if not method:continue
 if method=='initialize':
  result={'protocolVersion':1,'agentCapabilities':{'loadSession':True},'authMethods':[{'id':'cached_token'}] if mode!='no_auth_method' else [{'id':'browser'}],'_meta':{'agentVersion':'1.0.46' if mode!='version' else '99.0'}}
 elif method=='authenticate':
  assert p['methodId']=='cached_token' and p['_meta']['headless']
  if mode=='auth_error':send({'jsonrpc':'2.0','id':d['id'],'error':{'code':-1,'message':'not authenticated'}});continue
 elif method in ['session/new','session/load']:
  assert p['cwd']==os.getcwd() and p['mcpServers']==[]
  sid=p.get('sessionId') or str(uuid.uuid4())
  if method=='session/load':update({'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':'REPLAY_MUST_NOT_APPEAR'}})
  result={'sessionId':sid,'configOptions':[{'id':'model','currentValue':model},{'id':'reasoning_effort','currentValue':effort}]}
 elif method=='session/set_config_option':
  if p['configId']=='model':model=p['value']
  else:effort=p['value']
  result={'configOptions':[{'id':'model','currentValue':model if mode!='config' else 'wrong'},{'id':'reasoning_effort','currentValue':effort}]}
 elif method=='_x.ai/session/info':
  if mode=='pause_info' and calls==0:
   pause=pathlib.Path(synthetic['RRX_PAUSE']);pause.write_text('native preflight paused')
   while not pause.with_suffix('.continue').exists():time.sleep(0.01)
  if calls and mode=='late_write':assert fs('fs/write_text_file','late.txt','forbidden late effect').get('error')
  if calls and mode=='late_tool':tool('search_replace','late.txt',99)
  result={'result':{'sessionId':sid,'cwd':os.getcwd(),'agentName':'rururunx-decision' if decision else 'rururunx-executor','context':{'toolDefinitionsCount':(0 if decision else 2)+(1 if mode=='inventory' else 0),'toolCallCount':calls+(1 if calls and mode=='unnotified' else 0)}}}
 elif method=='session/prompt':
  if synthetic.get('RRX_PROMPT_OBSERVED'):pathlib.Path(synthetic['RRX_PROMPT_OBSERVED']).write_text('actual prompt received')
  prompt=p['_meta']['promptId'];assert 'bash_command' not in p['prompt'][0].get('_meta',{})
  assert p['prompt'][0]['text'].startswith('Prepared Task input follows:\n\n'), 'native slash command authority escaped envelope'
  if synthetic.get('RRX_EXPECT_INPUT'):assert p['prompt'][0]['text'].endswith(synthetic['RRX_EXPECT_INPUT'])
  send({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':sid,'update':{'sessionUpdate':'session_info_update','title':'native title'}}})
  connection=sqlite3.connect(synthetic['RRX_DATABASE']);rows=connection.execute("select body from records where kind='session'").fetchall()
  owned=[json.loads(row[0]) for row in rows if json.loads(row[0])['data']['recovery'].get('prompt_id')==prompt];assert len(owned)==1,'dispatch not durable'
  record=owned[0];recovery=record['data']['recovery'];assert recovery['input_version'] in [1,2,3]
  audit=connection.execute("select data from audit where kind='session.saved' order by sequence desc").fetchall();connection.close()
  saved=next(json.loads(row[0]) for row in audit if json.loads(row[0])['id']==record['id'])
  assert saved['evidence']['dispatch_intent']=={'input_version':recovery['input_version'],'prompt_id':prompt},'Grok dispatch intent was not atomically durable before wire'
  assert set(saved['evidence'])=={'state','agent','provider','role','native_ref','dispatch_intent'},'private recovery payload leaked into audit'
  if mode=='hang':time.sleep(60)
  if mode=='malformed':print('invalid-json',flush=True);continue
  if mode=='oversize':print('x'*1100000,flush=True);continue
  if mode=='invalid_callback_id':
   send({'jsonrpc':'2.0','id':None,'method':'fs/write_text_file','params':{'sessionId':sid,'path':'invalid-id.txt','content':'forbidden'}});json.loads(sys.stdin.readline())
  if mode=='config_update':update({'sessionUpdate':'config_option_update','configOptions':[]})
  if mode=='callback_budget':
   for _ in range(257):
    send({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{'sessionId':sid,'options':[{'kind':'reject_once','optionId':'deny'}]}});assert json.loads(sys.stdin.readline())['result']['outcome']['optionId']=='deny'
  if mode=='path_budget':fs('fs/read_text_file','a'*4097)
  if mode=='unknown_fs_method':assert fs('fs/unsupported_future','own.txt').get('error')
  if mode=='permission':
   send({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{'sessionId':sid,'options':[{'kind':'allow_once','optionId':'allow'},{'kind':'reject_once','optionId':'deny'}]}})
   answer=json.loads(sys.stdin.readline());assert answer['result']['outcome']['optionId']=='deny'
  if not decision:
   n=tool('search_replace' if mode=='wrong_method' else 'read_file','own.txt',1)
   if mode=='ambiguous':tool('read_file','own.txt',99)
   assert fs('fs/read_text_file','own.txt').get('result')
   if mode!='unfinished':done(n,mode=='failed_called_read')
   if mode=='ambiguous':done('99')
   if mode=='unowned_write':fs('fs/write_text_file','unowned.txt','unaccounted effect')
   if mode=='unowned_read':fs('fs/read_text_file','unseen.txt')
   if mode in ['supplemental_read','failed_called_read']:assert fs('fs/read_text_file','own.txt').get('result')
   if mode=='bypass':
    n=tool('search_replace','result.txt',2);done(n)
   else:
    target='own.txt' if mode=='replace_existing' else 'result.txt'
    n=tool('search_replace',target,2);fs('fs/read_text_file',target);assert fs('fs/write_text_file',target,'owned edit\n').get('result')=={};done(n)
   n=tool('search_replace',synthetic['RRX_FOREIGN'],3);assert fs('fs/write_text_file',synthetic['RRX_FOREIGN'],'forbidden').get('error');done(n,mode!='denied_completed')
  elif mode=='decision_tool':tool('read_file','own.txt',1)
  if mode in ['hook','hook_failure']:pathlib.Path('unexplained.txt').write_text('native hook effect')
  output={'verdict':'DENY','reason':'native fixture'}
  if mode=='schema_enum':output={'verdict':'ALLOW','reason':'native fixture'}
  elif mode=='schema_required':output={'verdict':'DENY'}
  elif mode=='schema_extra':output={'verdict':'DENY','reason':'native fixture','extra':True}
  result={'stopReason':'max_tokens' if mode=='hook_failure' else 'end_turn','_meta':{'sessionId':sid,'promptId':prompt,'usage':{'inputTokens':202 if 'explicit fresh continuation' in p['prompt'][0]['text'] else 101,'outputTokens':22 if 'explicit fresh continuation' in p['prompt'][0]['text'] else 11,'cachedReadTokens':0,'cacheCreationTokens':0},'structuredOutput':output}}
 send({'jsonrpc':'2.0','id':d['id'],'result':result})
